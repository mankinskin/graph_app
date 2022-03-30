use std::{sync::{
    RwLockReadGuard,
    RwLockWriteGuard,
}, ops::ControlFlow, borrow::Borrow};

use crate::{
    vertex::*,
    index::*,
    Hypergraph, HypergraphRef, Traversable, DirectedTraversalPolicy, QueryRangePath, StartPath, TraversableMut, TraversalNode, TraversalIterator, Dft, TraversalFolder,
};

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
        let start = start.indexing::<T, D, Self::Trav>(trav.clone());
        Self::parent_nodes(trav, query, Some(start))
    }
}
impl<T: Tokenize, D: IndexDirection> TraversalFolder<T, D> for Indexer<T, D> {
    type Trav = Self;
    type Break = Child;
    type Continue = Option<Child>;
    fn fold_found(
        trav: &Self::Trav,
        acc: Self::Continue,
        node: TraversalNode
    ) -> ControlFlow<Self::Break, Self::Continue> {
        match node {
            TraversalNode::End(Some(found)) => {
                ControlFlow::Break(match found.found {
                    FoundPath::Range(path) => {
                        let mut ltrav = trav.clone();
                        let (_, post) = ltrav.index_start_path(path.start);
                        post
                    },
                    FoundPath::Complete(c) => c
                } )
            },
            TraversalNode::Mismatch(_path, _query) => {
                unimplemented!()
            },
            _ => ControlFlow::Continue(acc)
        }
    }
}
pub(crate) trait IndexTraversable<T: Tokenize, D: IndexDirection>: TraversableMut<T> {
    fn index_start_path(&mut self, start: StartPath) -> (ChildLocation, Child) {
        let parent = start.entry().parent;
        let offset = parent.width - start.width();
        let (loc, _, post) = self.index_offset_split(parent, offset);
        (loc, post)
    }
    fn index_offset_split(&mut self, parent: Child, offset: usize) -> (ChildLocation, ContextHalf, Child) {
        let graph = self.graph();
        let child_patterns = graph.expect_children_of(parent).clone();
        let len = child_patterns.len();
        let perfect = child_patterns.into_iter()
            .try_fold(Vec::with_capacity(len), |mut acc, (pid, pattern)| {
                let (index, offset) = D::token_offset_split(pattern.borrow(), offset).unwrap();
                if offset == 0 {
                    ControlFlow::Break((pattern.into_pattern(), pid, index))
                } else {
                    acc.push((pattern.into_pattern(), index, offset));
                    ControlFlow::Continue(acc)
                }
            });
        drop(graph);
        match perfect {
            ControlFlow::Break((pattern, pid, pos)) => {
                let (pre, post) = D::directed_pattern_split(&pattern[..], pos);
                let post = if post.len() == 1 {
                    post.into_iter().next().unwrap()
                } else {
                    let mut graph = self.graph_mut();
                    let post = graph.insert_pattern(post);
                    graph.replace_in_pattern(parent, pid, pos.., [post]);
                    post
                };
                let pre = if pre.len() == 1 {
                    ContextHalf::Child(pre.into_iter().next().unwrap())
                } else {
                    ContextHalf::Pattern(pre)
                };
                (ChildLocation::new(parent, pid, pos), pre, post)
            },
            ControlFlow::Continue(positions) => {
                let (pres, posts) = positions.into_iter().map(|(pattern, pos, offset)| {
                    let (_, pre, post) = self.index_offset_split(*pattern.get(pos).unwrap(), offset);
                    (
                        [&D::back_context(pattern.borrow(), pos)[..], pre.borrow()].concat(),
                        [&[post], &D::front_context(pattern.borrow(), pos)[..]].concat(),
                    )
                }).unzip::<_, _, Vec<_>, Vec<_>>();
                println!("{:#?}", pres);
                println!("{:#?}", posts);
                let mut graph = self.graph_mut();
                let (pre, post) = (
                    graph.insert_patterns(pres),
                    graph.insert_patterns(posts),
                );
                let pid = graph.add_pattern_to_node(parent, [pre, post]);
                (ChildLocation::new(parent, pid, 1), ContextHalf::Child(pre), post)
            }
        }
    }
}
impl<T: Tokenize, D: IndexDirection> IndexTraversable<T, D> for Indexer<T, D> {}

impl<'g, T: Tokenize, D: IndexDirection> Indexer<T, D> {
    pub fn new(graph: HypergraphRef<T>) -> Self {
        Self {
            graph,
            _ty: Default::default(),
        }
    }
    pub(crate) fn index_pattern(
        &mut self,
        pattern: impl IntoPattern,
    ) -> Result<Child, NoMatch> {
        self.indexing::<Dft<_, _, _, _>, Indexing<T, D>, _>(pattern)
    }
    //pub(crate) fn index_prefix_at(
    //    &mut self,
    //    location: &ChildLocation,
    //) -> Result<Child, NoMatch> {
    //    self.graph_mut().index_range_in(location.parent, location.pattern_id, 0..location.sub_index + 1)
    //}
    //pub(crate) fn index_postfix_at(
    //    &mut self,
    //    location: &ChildLocation,
    //) -> Result<Child, NoMatch> {
    //    self.graph_mut().index_range_in(location.parent, location.pattern_id, location.sub_index..)
    //}
    //pub(crate) fn index_pre_context_at(
    //    &mut self,
    //    location: &ChildLocation,
    //) -> Result<Child, NoMatch> {
    //    self.graph_mut().index_range_in(location.parent, location.pattern_id, 0..location.sub_index)
    //}
    //pub(crate) fn index_post_context_at(
    //    &mut self,
    //    location: &ChildLocation,
    //) -> Result<Child, NoMatch> {
    //    self.graph_mut().index_range_in(location.parent, location.pattern_id, location.sub_index + 1..)
    //}
    //pub(crate) fn index_split(
    //    &mut self,
    //    path: ChildPath,
    //) -> IndexedChild {
    //    path.into_iter().fold(None, |acc, location| {
    //        let context = self.index_pre_context_at(&location).ok();
    //        Some(if let Some(IndexedChild {
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
    //            IndexedChild {
    //                context,
    //                inner: self.graph_mut().insert_pattern([prev_inner, inner]),
    //                location,
    //            }
    //        } else {
    //            IndexedChild {
    //                context,
    //                inner: self.index_postfix_at(&location).expect("Invalid child location!"),
    //                location,
    //            }
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
        let query_path = QueryRangePath::new_directed::<D, _>(query.borrow())?;

        match Ti::new(self, TraversalNode::Query(query_path))
            .try_fold(None, |acc, (_, node)|
                S::Folder::fold_found(self, acc, node)
            )
        {
            ControlFlow::Continue(None) => Err(NoMatch::NotFound(query)),
            ControlFlow::Continue(Some(found)) => Ok(found),
            ControlFlow::Break(found) => Ok(found)
        }
    }
}