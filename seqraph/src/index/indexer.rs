use std::{sync::{
    RwLockReadGuard,
    RwLockWriteGuard,
}, ops::ControlFlow};

use crate::{
    vertex::*,
    index::*,
    Hypergraph, HypergraphRef, Traversable, DirectedTraversalPolicy, QueryRangePath, StartPath, TraversableMut, TraversalNode, TraversalIterator, Bft, TraversalFolder,
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
trait IndexingTraversalPolicy<T: Tokenize, D: IndexDirection>:
    DirectedTraversalPolicy<T, D, Trav=Indexer<T, D>, Folder=Indexer<T, D>>
{}
impl<T: Tokenize, D: IndexDirection> IndexingTraversalPolicy<T, D> for Indexing<T, D> {}

impl<T: Tokenize, D: IndexDirection> DirectedTraversalPolicy<T, D> for Indexing<T, D> {
    type Trav = Indexer<T, D>;
    type Folder = Indexer<T, D>;
    fn end_op(
        trav: &Self::Trav,
        query: QueryRangePath,
        start: StartPath,
    ) -> Vec<TraversalNode> {
        let ltrav = trav.clone();
        ltrav.index_start_path(start);
        Self::parent_nodes(trav, query, Some(start))
    }
}
impl<T: Tokenize, D: IndexDirection> TraversalFolder<T, D> for Indexer<T, D> {
    type Trav = Self;
    type Break = Option<Child>;
    type Continue = Option<Child>;
    fn fold_found(
        trav: &Self::Trav,
        acc: Self::Continue,
        node: TraversalNode
    ) -> ControlFlow<Self::Break, Self::Continue> {
        match node {
            TraversalNode::End(found) => {
                ControlFlow::Break(found)
            },
            TraversalNode::Match(path, _, prev_query) => {
                let found = QueryFound::new(
                    path.reduce_end::<_, _, D>(trav),
                    prev_query,
                );
                if acc.as_ref().map(|f| found.found.gt(&f.found)).unwrap_or(true) {
                    ControlFlow::Continue(Some(found))
                } else {
                    ControlFlow::Continue(acc)
                }
            }
            _ => ControlFlow::Continue(acc)
        }
    }
}
impl<'g, T: Tokenize, D: IndexDirection> Indexer<T, D> {
    fn index_start_path(&mut self, start: StartPath) -> Child {
        let entry = start.entry();
        let pattern = self.graph().expect_pattern_at(&entry);
        let graph = self.graph();
        let child_patterns = graph.expect_children_of(entry.parent);
        let pattern = if let Some(last) = start.path().iter().next() {
            let last = graph.expect_child_at(last);
            let pattern = D::back_context(&pattern, entry.sub_index);
            if child_patterns.len() < 2 {
                return self.graph_mut().insert_pattern([&[last], &pattern[..]].concat());
            } else {
                pattern
            }
        } else {
            let pattern = D::split_end(&pattern, entry.sub_index);
            if child_patterns.len() < 2 {
                return self.graph_mut().insert_pattern(pattern);
            } else {
                pattern
            }
        };
        std::iter::once(pattern)
            .chain(
                child_patterns.iter()
                    .filter(|(pid, _)| pid != entry.pattern_id)
                    .map(|(_, pattern)| self.index_offset_split(pattern, start.width))
            )
    }
    fn index_offset_split(&mut self, pattern: Pattern, width: usize) -> (Child, Child) {
        let (index, offset) = <D as MatchDirection>::Opposite::token_offset_split(pattern, width).unwrap();
        if offset == 0 {
            let (post, pre) = <D as MatchDirection>::Opposite::directed_pattern_split(pattern, index);
            let graph = self.graph_mut();
            (
                graph.insert_pattern(pre),
                graph.insert_pattern(post),
            )
        } else {

        }
    }
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
        pattern: impl IntoPattern<Token=impl AsChild + Clone + Vertexed<'a, 'g>>,
    ) -> Result<Child, NoMatch> {
        self.indexing::<Bft<_, _, _, _>, Indexing<T, D>, _>(pattern)
    }
    //pub(crate) fn index_split(
    //    &mut self,
    //    path: ChildPath,
    //) -> IndexedChild {
    //    path.into_iter().fold(None, |acc, location| {
    //        let context = self.index_pre_context_at(&location).ok();
    //        let (inner, context, location) = if let Some(IndexedChild {
    //                context: prev_context,
    //                inner: prev_inner,
    //                ..
    //            }) = acc {
    //            let context = context.and_then(|context|
    //                prev_context.map(|prev_context|
    //                    self.graph_mut().insert_pattern([context, prev_context].as_slice())
    //                )
    //                .or(Some(context))
    //            )
    //            .or(prev_context);
    //            let inner = self.index_post_context_at(&location).expect("Invalid child location!");
    //            (
    //                self.graph_mut().insert_pattern([prev_inner, inner]),
    //                context,
    //                location,
    //            )
    //        } else {
    //            (
    //                self.index_postfix_at(&location).expect("Invalid child location!"),
    //                context,
    //                location,
    //            )
    //        };
    //        Some(IndexedChild {
    //            context,
    //            inner,
    //            location,
    //        })
    //    })
    //    .unwrap()
    //}
    /// creates an IndexingNode::Parent for each parent of root, extending its start path
    fn indexing<
        'a,
        Ti: TraversalIterator<'g, T, Self, D, S>,
        S: IndexingTraversalPolicy<T, D>,
        Q: IntoPattern,
    >(
        &'g mut self,
        query: Q,
    ) -> Result<Child, NoMatch> {
        let query = query.into_pattern();
        let query_path = QueryRangePath::new_directed::<D, _>(query.as_pattern_view())?;

        match Ti::new(self, TraversalNode::Query(query_path))
            .try_fold(None, |acc, (_, node)|
                S::Folder::fold_found(self, acc, node)
            )
        {
            ControlFlow::Continue(None) => Err(NoMatch::NotFound(query)),
            ControlFlow::Continue(Some(found)) => Ok(found),
            ControlFlow::Break(found) => found.ok_or(NoMatch::SingleIndex)
        }
    }
}