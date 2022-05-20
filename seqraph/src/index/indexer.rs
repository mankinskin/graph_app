use std::{
    sync::{
        RwLockReadGuard,
        RwLockWriteGuard,
    },
};

use crate::{
    *,
    vertex::*,
    index::*,
    Hypergraph,
    HypergraphRef,
    DirectedTraversalPolicy,
    QueryRangePath,
    StartPath,
    TraversableMut,
    TraversalNode,
    TraversalIterator,
    TraversalFolder,
    Bft, EndPath, IndexingPath, DirectedBorderPath,
};

#[derive(Debug, Clone)]
pub struct Indexer<T: Tokenize, D: IndexDirection> {
    graph: HypergraphRef<T>,
    _ty: std::marker::PhantomData<D>,
}
struct Indexing<'a, T: Tokenize, D: IndexDirection, Q: IndexingQuery> {
    _ty: std::marker::PhantomData<(&'a T, D, Q)>,
}
impl<'a: 'g, 'g, T: Tokenize + 'a, D: IndexDirection> Traversable<'a, 'g, T> for Indexer<T, D> {
    type Guard = RwLockReadGuard<'g, Hypergraph<T>>;
    fn graph(&'g self) -> Self::Guard {
        self.graph.read().unwrap()
    }
}
impl<'a: 'g, 'g, T: Tokenize + 'a, D: IndexDirection> TraversableMut<'a, 'g, T> for Indexer<T, D> {
    type GuardMut = RwLockWriteGuard<'g, Hypergraph<T>>;
    fn graph_mut(&'g mut self) -> Self::GuardMut {
        self.graph.write().unwrap()
    }
}
impl<'a: 'g, 'g, T: Tokenize, D: IndexDirection, Q: IndexingQuery> DirectedTraversalPolicy<'a, 'g, T, D, Q> for Indexing<'a, T, D, Q> {
    type Trav = Indexer<T, D>;
    type Folder = Indexer<T, D>;
    fn after_match_end(
        trav: &'a Self::Trav,
        path: SearchPath,
    ) -> FolderStartPath<'a, 'g, T, D, Q, Self> {
        let mut ltrav = trav.clone();
        let IndexSplitResult {
            inner: post,
            location: entry,
            ..
            // should call leaf split and use known info of leaf position
        } = SideIndexable::<_, D, IndexBack>::index_entry_split(&mut ltrav, path.get_entry_location(), path.width());
        StartLeaf { entry, child: post, width: path.width() }
    }
}
trait IndexingTraversalPolicy<'a: 'g, 'g, T: Tokenize, D: IndexDirection, Q: IndexingQuery>:
    DirectedTraversalPolicy<'a, 'g, T, D, Q, Trav=Indexer<T, D>, Folder=Indexer<T, D>>
{ }
impl<'a: 'g, 'g, T: Tokenize, D: IndexDirection, Q: IndexingQuery> IndexingTraversalPolicy<'a, 'g, T, D, Q> for Indexing<'a, T, D, Q> {}

pub(crate) trait IndexingQuery: TraversalQuery + ReduciblePath {}
impl<T: TraversalQuery + ReduciblePath> IndexingQuery for T {}

impl<'a: 'g, 'g, T: Tokenize + 'a, D: IndexDirection, Q: IndexingQuery> TraversalFolder<'a, 'g, T, D, Q> for Indexer<T, D> {
    type Trav = Self;
    type Break = (Child, Q);
    type Continue = Option<TraversalResult<IndexingPath, Q>>;
    type Path = IndexingPath;
    type StartPath = StartLeaf;
    type Node = IndexingNode<Q>;
    fn fold_found(
        trav: &Self::Trav,
        acc: Self::Continue,
        node: Self::Node,
    ) -> ControlFlow<Self::Break, Self::Continue> {
        let mut trav = trav.clone();
        match node {
            IndexingNode::End(Some(found)) => {
                ControlFlow::Break((
                    Indexable::<_, D>::index_found(&mut trav, found.found),
                    found.query,
                ))
            },
            IndexingNode::Mismatch(paths) => {
                Indexable::<_, D>::index_mismatch(&mut trav, acc, paths)
            },
            IndexingNode::Match(path, _, prev_query) => {
                let found = TraversalResult::new(
                    path.reduce_end::<_, D, _>(&trav),
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
pub(crate) trait Indexable<'a: 'g, 'g, T: Tokenize, D: IndexDirection>: TraversableMut<'a, 'g, T> {
    #[named]
    fn index_found(
        &'a mut self,
        found: IndexingFoundPath,
    ) -> Child {
        trace!(function_name!());
        match found {
            FoundPath::Range(path) => self.index_range_path(path),
            FoundPath::Complete(c) => c
        }
    }
    fn index_end_path(
        &'a mut self,
        end: EndPath,
    ) -> IndexSplitResult {
        let EndPath {
            mut path,
            entry,
            width,
        } = end;
        let mut graph = self.graph_mut();
        while let Some(entry) = path.pop() {
            SideIndexable::<_, D, IndexFront>::index_entry_split(
                &mut *graph,
                entry,
                width,
            );
        }
        SideIndexable::<_, D, IndexFront>::index_entry_split(
            &mut *graph,
            entry,
            width,
        )
    }
    #[named]
    fn index_mismatch<Acc, Q: TraversalQuery + ReduciblePath>(
        &'a mut self,
        acc: Acc,
        paths: PathPair<Q, IndexingPath>,
    ) -> ControlFlow<(Child, Q), Acc> {
        trace!(function_name!());
        let mut graph = self.graph_mut();
        let found = paths.reduce_mismatch::<_, D, _>(&*graph);
        if let FoundPath::Range(path) = &found.found {
            if path.get_exit_pos() == path.get_entry_pos() {
                return ControlFlow::Continue(acc);
            }
        }
        ControlFlow::Break((
            Indexable::<_, D>::index_found(&mut *graph, found.found),
            found.query
        ))
    }
    #[named]
    fn index_range_path(
        &'a mut self,
        path: IndexingPath,
    ) -> Child {
        trace!(function_name!());
        let offset = path.width();
        let location@ChildLocation {
            parent,
            sub_index: entry,
            ..
        } = path.start.entry();
        let exit = path.get_exit_pos();
        let inner_width = path.inner_width;
        let (start, end) = path.into_paths();
        let mut graph = self.graph_mut();
        let pattern = start.pattern(&*graph);
        match (
            start.is_perfect(),
            DirectedBorderPath::<D>::is_at_pattern_border(&start, pattern.borrow()),
            end.is_perfect(),
            DirectedBorderPath::<D>::is_at_pattern_border(&end, pattern.borrow()),
        ) {
            //   start         end
            // perf comp    perf   comp
            (true, true, true, true) => panic!("IndexingPath references complete index!"),
            // perfect back half split
            (true, _, true, true) =>
                SideIndexable::<_, D, IndexBack>::pattern_index_perfect_split(&mut *graph, pattern, location),
            // perfect front half split
            (true, true, true, _) =>
                SideIndexable::<_, D, IndexFront>::pattern_index_perfect_split(&mut *graph, pattern, end.entry()),
            // unperfect back half split
            (false, _, true, true) =>
                SideIndexable::<_, D, IndexBack>::index_offset_split(
                    &mut *graph,
                    parent,
                    <IndexBack as IndexSide<D>>::width_offset(&parent, offset)
                ),
            // unperfect front half split
            (true, true, false, _) =>
                SideIndexable::<_, D, IndexFront>::index_offset_split(
                    &mut *graph,
                    parent,
                    <IndexFront as IndexSide<D>>::width_offset(&parent, offset)
                ),
            // perfect/perfect inner split
            (true, _, true, _) =>
                Indexable::<_, D>::pattern_index_perfect_split_range(&mut *graph, pattern, location, entry..=exit),
            // unperfect/perfect inner split
            (false, _, true, false) =>
                SideIndexable::<_, D, IndexBack>::pattern_range_unperfect_split(
                    &mut *graph,
                    pattern,
                    location,
                    <IndexBack as IndexSide<D>>::width_offset(&parent, offset),
                    <IndexBack as IndexSide<D>>::limited_range(entry, exit),
                ),
            // perfect/unperfect inner split
            (true, false, false, _) =>
                SideIndexable::<_, D, IndexFront>::pattern_range_unperfect_split(
                    &mut *graph,
                    pattern,
                    location,
                    <IndexFront as IndexSide<D>>::width_offset(&parent, offset),
                    <IndexFront as IndexSide<D>>::limited_range(entry, exit),
                ),
            // unperfect/unperfect inner split
            (false, _, false, _) => {
                let child_patterns = graph.expect_children_of(parent).clone();

                let pattern = start.entry().expect_pattern_in(&child_patterns);
                let pre_width = pattern[..start.entry().sub_index].width();
                let back_index = start.entry().expect_child_in_pattern(pattern);
                let back_offset = pre_width + <IndexBack as IndexSide<D>>::width_offset(back_index, start.width());
                let front_index = end.entry().expect_child_in_pattern(pattern);
                let front_offset = pre_width + back_index.width() + inner_width + <IndexFront as IndexSide<D>>::width_offset(front_index, end.width());
                let positions = child_patterns.into_iter()
                    .map(|(pid, pattern)| {
                        let (back_index, back_offset) = <IndexBack as IndexSide<D>>::token_offset_split(pattern.borrow(), back_offset).unwrap();
                        let (front_index, front_offset) = <IndexFront as IndexSide<D>>::token_offset_split(pattern.borrow(), front_offset).unwrap();
                        (pid, pattern.into_pattern(), back_index, back_offset, front_index, front_offset)
                    })
                    .collect_vec();
                let (backs, inners, fronts): (Vec<_>, Vec<_>, Vec<_>) = multiunzip(positions.into_iter()
                    .map(|(_, pattern, back_pos, back_offset, front_pos, front_offset)| {
                        let IndexSplitResult {
                            inner: back_inner,
                            context: back_context_path,
                            location: back_location,
                        } = SideIndexable::<_, D, IndexBack>::index_offset_split(&mut *graph, *pattern.get(back_pos).unwrap(), back_offset);
                        let back_context = SideIndexable::<_, D, IndexBack>::index_context_path(&mut *graph, back_location, back_context_path);

                        let IndexSplitResult {
                            inner: front_inner,
                            context: front_context_path,
                            location: front_location,
                        } = SideIndexable::<_, D, IndexFront>::index_offset_split(&mut *graph, *pattern.get(front_pos).unwrap(), front_offset);
                        let front_context = SideIndexable::<_, D, IndexFront>::index_context_path(&mut *graph, front_location, front_context_path);

                        let inner_context = pattern.get(D::inner_context_range(back_pos, front_pos)).and_then(|p| graph.insert_pattern(p));
                        let inner_context = inner_context.as_ref()
                            .map(std::slice::from_ref)
                            .unwrap_or_default();
                        (
                            // todo: order depends on D
                            [&D::back_context(pattern.borrow(), back_pos)[..], &[back_context]].concat(),
                            [&[back_inner], inner_context, &[front_inner]].concat(),
                            [&[front_context], &D::front_context(pattern.borrow(), front_pos)[..]].concat(),
                        )
                    }));
                let (back, inner, front) = (
                    graph.index_patterns(backs),
                    graph.index_patterns(inners),
                    graph.index_patterns(fronts),
                );
                let pid = graph.add_pattern_with_update(parent, [back, inner, front]);
                let location = ChildLocation::new(parent, pid, 1);
                IndexSplitResult {
                    location,
                    context: vec![],
                    inner,
                }
            },
        }.inner
    }
    #[named]
    fn pattern_index_perfect_split_range(
        &'a mut self,
        pattern: Pattern,
        location: impl IntoPatternLocation,
        range: impl PatternRangeIndex + StartInclusive,
    ) -> IndexSplitResult {
        trace!(function_name!());
        let inner = &pattern[range.clone()];
        let location = location.into_pattern_location().to_child_location(range.start());
        let inner = if inner.len() == 1 {
            *inner.iter().next().unwrap()
        } else {
            let mut graph = self.graph_mut();
            let inner = graph.index_pattern(inner);
            graph.replace_in_pattern(&location, range, [inner]);
            inner
        };
        IndexSplitResult {
            location,
            context: vec![],
            inner,
        }
    }
}
impl<
    'a: 'g,
    'g,
    T: Tokenize,
    D: IndexDirection,
    Trav: TraversableMut<'a, 'g, T>,
> Indexable<'a, 'g, T, D> for Trav {}


impl<'a: 'g, 'g, T: Tokenize, D: IndexDirection> Indexer<T, D> {
    pub fn new(graph: HypergraphRef<T>) -> Self {
        Self {
            graph,
            _ty: Default::default(),
        }
    }
    pub(crate) fn index_prefix(
        &mut self,
        pattern: impl IntoPattern,
    ) -> Result<(Child, QueryRangePath), NoMatch> {
        self.indexing::<Bft<_, _, _, _, _>, Indexing<T, D, QueryRangePath>, _>(pattern)
    }
    /// creates an IndexingNode::Parent for each parent of root, extending its start path
    fn indexing<
        Ti: TraversalIterator<'a, 'g, T, Self, D, QueryRangePath, S>,
        S: IndexingTraversalPolicy<'a, 'g, T, D, QueryRangePath>,
        P: IntoPattern,
    >(
        &'a mut self,
        query: P,
    ) -> Result<(Child, QueryRangePath), NoMatch> {
        let query = query.into_pattern();
        let query_path = QueryRangePath::new_directed::<D, _>(query.borrow())?;
        self.path_indexing::<_, Ti, S>(query_path)
    }
    pub(crate) fn index_path_prefix<
        Q: IndexingQuery,
    >(
        &mut self,
        query: Q,
    ) -> Result<(Child, Q), NoMatch> {
        self.path_indexing::<_, Bft<_, _, _, _, _>, Indexing<T, D, Q>>(query)
    }
    fn path_indexing<
        Q: IndexingQuery,
        Ti: TraversalIterator<'a, 'g, T, Self, D, Q, S>,
        S: IndexingTraversalPolicy<'a, 'g, T, D, Q>,
    >(
        &'a mut self,
        query_path: Q,
    ) -> Result<(Child, Q), NoMatch> {
        let result = Ti::new(self, TraversalNode::query_node(query_path))
            .try_fold(None, |acc, (_, node)|
                S::Folder::fold_found(self, acc, node)
            );
        match result {
            ControlFlow::Continue(None) => Err(NoMatch::NotFound),
            ControlFlow::Continue(Some(found)) => Ok((Indexable::<_, D>::index_found(&mut self.clone(), found.found), found.query)),
            ControlFlow::Break((found, query)) => Ok((found, query))
        }
    }
}