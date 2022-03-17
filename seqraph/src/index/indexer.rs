use std::{sync::{
    RwLockReadGuard,
    RwLockWriteGuard,
}, ops::ControlFlow};

use crate::{
    vertex::*,
    index::*,
    Hypergraph, HypergraphRef,
};
use itertools::*;

#[derive(Debug, Clone)]
pub struct Indexer<T: Tokenize, D: IndexDirection> {
    graph: HypergraphRef<T>,
    _ty: std::marker::PhantomData<D>,
}
struct Indexing<T: Tokenize, D: IndexDirection> {
    _ty: std::marker::PhantomData<(T, D)>,
}
impl<T: Tokenize, D: IndexDirection> Traversable<T> for Indexer<T, D> {
    fn graph(&self) -> RwLockReadGuard<'_, Hypergraph<T>> {
        self.graph.read().unwrap()
    }
}
impl<T: Tokenize, D: IndexDirection> TraversableMut<T> for Indexer<T, D> {
    fn graph_mut(&mut self) -> RwLockWriteGuard<'_, Hypergraph<T>> {
        self.graph.write().unwrap()
    }
}
impl<'g, T: Tokenize, D: IndexDirection + 'g> DirectedTraversalPolicy<'g, T, D> for Indexing<T, D> {
    type Trav = Indexer<T, D>;
    fn end_op(
        trav: Self::Trav,
        query: QueryRangePath,
        start: StartPath,
    ) -> Vec<TraversalNode> {
        // root at end of parent, recurse upwards to get all next children
        Self::parent_nodes(trav, query, Some(start))
    }
}
impl<'g, T: Tokenize, D: IndexDirection> Indexer<T, D> {
    pub fn new(graph: HypergraphRef<T>) -> Self {
        Self {
            graph,
            _ty: Default::default(),
        }
    }
    /// includes location
    #[allow(unused)]
    pub(crate) fn index_prefix_at(
        &mut self,
        location: &ChildLocation,
    ) -> Result<Child, NoMatch> {
        self.graph_mut().index_range_in(location.parent, location.pattern_id, 0..location.sub_index + 1)
    }
    /// includes location
    pub(crate) fn index_postfix_at(
        &mut self,
        location: &ChildLocation,
    ) -> Result<Child, NoMatch> {
        self.graph_mut().index_range_in(location.parent, location.pattern_id, location.sub_index..)
    }
    /// does not include location
    pub(crate) fn index_pre_context_at(
        &mut self,
        location: &ChildLocation,
    ) -> Result<Child, NoMatch> {
        self.graph_mut().index_range_in(location.parent, location.pattern_id, 0..location.sub_index)
    }
    /// does not include location
    pub(crate) fn index_post_context_at(
        &mut self,
        location: &ChildLocation,
    ) -> Result<Child, NoMatch> {
        self.graph_mut().index_range_in(location.parent, location.pattern_id, location.sub_index + 1..)
    }
    pub(crate) fn index_pattern<'a>(
        &'g mut self,
        pattern: impl IntoPattern<Item = impl AsChild, Token=impl AsChild + Clone + Vertexed<'a, 'g>>,
    ) -> Result<Child, NoMatch> {
        self.indexing(pattern)
    }
    pub(crate) fn index_split(
        &mut self,
        path: ChildPath,
    ) -> IndexedChild {
        path.into_iter().fold(None, |acc, location| {
            let context = self.index_pre_context_at(&location).ok();
            let (inner, context, location) = if let Some(IndexedChild {
                    context: prev_context,
                    inner: prev_inner,
                    ..
                }) = acc {
                let context = context.and_then(|context|
                    prev_context.map(|prev_context|
                        self.graph_mut().insert_pattern([context, prev_context].as_slice())
                    )
                    .or(Some(context))
                )
                .or(prev_context);
                let inner = self.index_post_context_at(&location).expect("Invalid child location!");
                (
                    self.graph_mut().insert_pattern([prev_inner, inner]),
                    context,
                    location,
                )
            } else {
                (
                    self.index_postfix_at(&location).expect("Invalid child location!"),
                    context,
                    location,
                )
            };
            Some(IndexedChild {
                context,
                inner,
                location,
            })
        })
        .unwrap()
    }
    fn indexing<
        'a,
        C: AsChild,
        Q: IntoPattern<Item = C>,
    >(
        &mut self,
        query: Q,
    ) -> Result<Child, NoMatch> {
        self.bft_indexing(query)?
            .into_iter()
            // collect paths into subgraph
            .fold(None, |acc: Option<IndexedPath>, indexed| {
                if let Some(acc) = acc {
                    assert!(
                        acc.indexed.location.parent == indexed.indexed.location.parent,
                        "Found multiple roots!"
                    );
                    Some(acc)
                } else {
                    Some(indexed)
                }
            })
            .map(|indexed| indexed.indexed.location.parent)
            .ok_or_else(|| NoMatch::NotFound(query.into_pattern()))
    }
    /// creates an IndexingNode::Parent for each parent of root, extending its start path
    fn bft_indexing<
        'a,
        C: AsChild,
        Q: IntoPattern<Item = C>,
    >(
        &mut self,
        query: Q,
    ) -> Result<Vec<IndexedPath>, NoMatch> {
        // try any parent, i.e. first
        let query = query.into_pattern();
        let query_path = QueryRangePath::new_directed::<D, _, _>(&query)?;
        // if context not empty
        // breadth first traversal
        //let subgraph = Arc::new(RwLock::new(Subgraph::new()));
        let indexer = self.clone();
        let mut indexer2 = self.clone();

        // breadth first iterator over graph from start
        match Bft::new(TraversalNode::Query(query_path), |node| {
            match node.clone() {
                TraversalNode::Query(query) =>
                    // search parents of start
                    Indexing::parent_nodes(
                        indexer.clone(),
                        query,
                        None,
                    ),
                TraversalNode::Root(query, start_path) =>
                    Indexing::root_successor_nodes(
                        indexer.clone(),
                        query,
                        start_path,
                    ),
                TraversalNode::Match(path, query) =>
                    Indexing::match_next(
                        indexer.clone(),
                        path,
                        query,
                    ),
                _ => vec![],
            }.into_iter()
        })
        // iterator over all roots with paths from start to query_next
        .try_fold(None, |acc: Option<QueryFound>, (_, node)|
            fold_found(acc, node)
        ) {
            ControlFlow::Continue(None) => Err(NoMatch::NotFound(query)),
            ControlFlow::Continue(Some(found)) => Ok(found),
            ControlFlow::Break(found) => Ok(found)
        }
    }
}