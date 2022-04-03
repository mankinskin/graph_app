use std::{sync::{
    RwLockReadGuard,
    RwLockWriteGuard,
}, ops::{ControlFlow, RangeInclusive, RangeFrom}, borrow::Borrow};

use crate::{
    vertex::*,
    index::*,
    Hypergraph,
    HypergraphRef,
    Traversable,
    DirectedTraversalPolicy,
    QueryRangePath,
    StartPath,
    TraversableMut,
    TraversalNode,
    TraversalIterator,
    TraversalFolder,
    Bft, EndPath, GraphPath,
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
        let mut ltrav = trav.clone();
        let (loc, post) = IndexFront::index_path(&mut ltrav, start);
        let start = StartPath::First(loc, post, post.width());
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
                        let (_, front) = IndexFront::index_path(&mut ltrav, path.into_start_path());
                        //let (_, back) = ltrav.index_end_path(path);
                        front
                    },
                    FoundPath::Complete(c) => c
                } )
            },
            TraversalNode::Mismatch(path) => {
                let found = path.reduce_mismatch::<_, _, D>(trav);
                match found {
                    FoundPath::Range(path) =>
                        if path.has_end_match::<_, _, D>(trav) {
                            let mut ltrav = trav.clone();
                            let (_, front) = IndexFront::index_path(&mut ltrav, path.into_start_path());
                            //let (_, back) = ltrav.index_end_path(path);
                            ControlFlow::Break(front)
                        } else {
                            ControlFlow::Continue(acc)
                        },
                    FoundPath::Complete(c) => ControlFlow::Break(c)
                }
            },
            _ => ControlFlow::Continue(acc)
        }
    }
}
pub(crate) struct IndexSplitResult {
    inner: Child,
    context: ContextHalf,
    location: ChildLocation,
}
pub(crate) trait IndexSide<T: Tokenize, D: IndexDirection> {
    type Trav: TraversableMut<T>;
    type Path: GraphPath;
    type Range: PatternRangeIndex;
    fn width_offset(path: &Self::Path) -> usize;
    fn back_front_order<A>(back: A, front: A) -> IndexingPair<A>;
    fn context_inner_order<
        'a,
        C: AsRef<[Child]> + 'a,
        I: AsRef<[Child]> + 'a
    >(context: &'a C, inner: &'a I) -> (&'a [Child], &'a [Child]);
    fn replace_range(pos: usize) -> Self::Range;
    fn index_path(trav: &mut Self::Trav, path: Self::Path) -> (ChildLocation, Child) {
        let offset = Self::width_offset(&path);
        let parent = path.entry().parent;
        let IndexSplitResult {
            location,
            inner,
            ..
        } = Self::index_offset_split(trav, parent, offset);
        (location, inner)
    }
    fn index_offset_split(trav: &mut Self::Trav, parent: Child, offset: usize) -> IndexSplitResult {
        let graph = trav.graph();
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
            // perfect split
            ControlFlow::Break((pattern, pid, pos)) => {
                let (back, front) = D::directed_pattern_split(&pattern[..], pos);
                let IndexingPair {
                    inner,
                    context
                } = Self::back_front_order(back, front);
                let inner = if inner.len() == 1 {
                    inner.into_iter().next().unwrap()
                } else {
                    let mut graph = trav.graph_mut();
                    let inner = graph.insert_pattern(inner);
                    graph.replace_in_pattern(parent, pid, Self::replace_range(pos), [inner]);
                    inner
                };
                let context = if context.len() == 1 {
                    ContextHalf::Child(context.into_iter().next().unwrap())
                } else {
                    ContextHalf::Pattern(context)
                };
                IndexSplitResult {
                    location: ChildLocation::new(parent, pid, pos),
                    context,
                    inner,
                }
            },
            // no perfect split
            ControlFlow::Continue(positions) => {
                let (backs, fronts) = positions.into_iter()
                    .map(|(pattern, pos, offset)| {
                        let IndexSplitResult {
                            inner,
                            context,
                            ..
                        } = Self::index_offset_split(trav, *pattern.get(pos).unwrap(), offset);
                        let (back, front) = Self::context_inner_order(&context, &inner);
                        (
                            // todo: order depends on D
                            [&D::back_context(pattern.borrow(), pos)[..], back].concat(),
                            [front, &D::front_context(pattern.borrow(), pos)[..]].concat(),
                        )
                    }).unzip::<_, _, Vec<_>, Vec<_>>();
                let mut graph = trav.graph_mut();
                let (back, front) = (
                    graph.insert_patterns(backs),
                    graph.insert_patterns(fronts),
                );
                let pid = graph.add_pattern_to_node(parent, [back, front]);
                let IndexingPair {
                    inner,
                    context
                } = Self::back_front_order(back, front);
                IndexSplitResult {
                    location: ChildLocation::new(parent, pid, 1),
                    context: ContextHalf::Child(context),
                    inner,
                }
            }
        }
    }
}
pub(crate) struct IndexingPair<T> {
    inner: T,
    context: T,
}
pub(crate) struct IndexFront;
impl<T: Tokenize, D: IndexDirection> IndexSide<T, D> for IndexFront {
    type Trav = Indexer<T, D>;
    type Path = StartPath;
    type Range = RangeFrom<usize>;
    fn replace_range(pos: usize) -> Self::Range {
        pos..
    }
    fn context_inner_order<
        'a,
        C: AsRef<[Child]> + 'a,
        I: AsRef<[Child]> + 'a
    >(context: &'a C, inner: &'a I) -> (&'a [Child], &'a [Child]) {
        (context.as_ref(), inner.as_ref())
    }
    fn back_front_order<A>(back: A, front: A) -> IndexingPair<A> {
        IndexingPair {
            inner: front,
            context: back,
        }
    }
    fn width_offset(path: &Self::Path) -> usize {
        // todo: changes with index direction
        path.entry().parent.width() - path.width()
    }
}
pub(crate) struct IndexBack;
impl<T: Tokenize, D: IndexDirection> IndexSide<T, D> for IndexBack {
    type Trav = Indexer<T, D>;
    type Path = EndPath;
    type Range = RangeInclusive<usize>;
    fn replace_range(pos: usize) -> Self::Range {
        0..=pos
    }
    fn context_inner_order<
        'a,
        C: AsRef<[Child]> + 'a,
        I: AsRef<[Child]> + 'a
    >(context: &'a C, inner: &'a I) -> (&'a [Child], &'a [Child]) {
        (inner.as_ref(), context.as_ref())
    }
    fn back_front_order<A>(back: A, front: A) -> IndexingPair<A> {
        IndexingPair {
            inner: back,
            context: front,
        }
    }
    fn width_offset(path: &Self::Path) -> usize {
        path.width()
    }
}

impl<'g, T: Tokenize, D: IndexDirection> Indexer<T, D> {
    pub fn new(graph: HypergraphRef<T>) -> Self {
        Self {
            graph,
            _ty: Default::default(),
        }
    }
    pub(crate) fn index_prefix(
        &mut self,
        pattern: impl IntoPattern,
    ) -> Result<Child, NoMatch> {
        self.indexing::<Bft<_, _, _, _>, Indexing<T, D>, _>(pattern)
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