use std::{sync::{
    RwLockReadGuard,
    RwLockWriteGuard,
}, ops::ControlFlow, borrow::Borrow};

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
        let mut ltrav = trav.clone();
        let start = ltrav.index_start_path(start);
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
                        let (_, _, post) = ltrav.index_offset_split(
                            path.start.entry().parent,
                            path.start.width(),
                        );
                        post
                    },
                    FoundPath::Complete(c) => c
                } )
            },
            TraversalNode::Mismatch(path, query) => {
                unimplemented!()
            },
            _ => ControlFlow::Continue(acc)
        }
    }
}
enum PatternSplitPosition {
    Perfect(Pattern, PatternId, usize),
    Imperfect(Pattern, usize, usize),
}
enum ContextHalf {
    Child(Child),
    Pattern(Pattern),
}
impl Borrow<[Child]> for ContextHalf {
    fn borrow(&self) -> &[Child] {
        match self {
            Self::Child(c) => std::slice::from_ref(c),
            Self::Pattern(p) => p.borrow(),
        }
    }
}
impl<'g, T: Tokenize, D: IndexDirection> Indexer<T, D> {
    fn index_start_path(&mut self, start: StartPath) -> StartPath {
        let (loc, _, post) = self.index_offset_split(start.entry().parent, start.width());
        StartPath::First(loc, post, start.width())
    }
    fn index_offset_split(&mut self, parent: Child, width: usize) -> (ChildLocation, ContextHalf, Child) {
        let graph = self.graph();
        let child_patterns = graph.expect_children_of(parent).clone();
        let len = child_patterns.len();
        let perfect = child_patterns.into_iter()
            .try_fold(Vec::with_capacity(len), |mut acc, (pid, pattern)| {
                let (index, offset) = <D as MatchDirection>::Opposite::token_offset_split(pattern.borrow(), width).unwrap();
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
                let (post, pre) = <D as MatchDirection>::Opposite::directed_pattern_split(&pattern[..], pos);
                let mut graph = self.graph_mut();
                let post = graph.insert_pattern(post);
                graph.replace_in_pattern(parent, pid, pos.., [post]);
                let pre = if pre.len() == 1 {
                    ContextHalf::Child(pre.into_iter().next().unwrap())
                } else {
                    ContextHalf::Pattern(pre)
                };
                (ChildLocation::new(parent, pid, pos), pre, post)
            },
            ControlFlow::Continue(positions) => {
                let (pre, post) = positions.into_iter().map(|(pattern, pos, offset)| {
                    let (_, pre, post) = self.index_offset_split(*pattern.get(pos).unwrap(), offset);
                    (
                        [&D::front_context(pattern.borrow(), pos)[..], pre.borrow()].concat(),
                        [&[post], &D::back_context(pattern.borrow(), pos)[..]].concat(),
                    )
                }).unzip::<_, _, Vec<_>, Vec<_>>();
                let mut graph = self.graph_mut();
                let (pre, post) = (
                    graph.insert_patterns(pre),
                    graph.insert_patterns(post),
                );
                let pid = graph.add_pattern_to_node(parent, [pre, post]);
                (ChildLocation::new(parent, pid, 1), ContextHalf::Child(pre), post)
            }
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
        pattern: impl IntoPattern,
    ) -> Result<Child, NoMatch> {
        self.indexing::<Bft<_, _, _, _>, Indexing<T, D>, _>(pattern)
    }
    pub(crate) fn index_split(
        &mut self,
        path: ChildPath,
    ) -> IndexedChild {
        path.into_iter().fold(None, |acc, location| {
            let context = self.index_pre_context_at(&location).ok();
            Some(if let Some(IndexedChild {
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
                IndexedChild {
                    context,
                    inner: self.graph_mut().insert_pattern([prev_inner, inner]),
                    location,
                }
            } else {
                IndexedChild {
                    context,
                    inner: self.index_postfix_at(&location).expect("Invalid child location!"),
                    location,
                }
            })
        })
        .unwrap()
    }
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