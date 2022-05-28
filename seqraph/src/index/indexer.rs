use crate:: *;
use super::*;

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
        } = SideIndexable::<_, D, IndexBack>::entry_split(
            &mut ltrav,
            path.get_entry_location(),
            path.width()
        );
        StartPath::Leaf(StartLeaf { entry, child: post, width: path.width() })
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
    type Continue = Option<TraversalResult<SearchPath, Q>>;
    type Path = SearchPath;
    type StartPath = StartPath;
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
    fn index_found(
        &'a mut self,
        found: SearchFoundPath,
    ) -> Child {
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
            SideIndexable::<_, D, IndexFront>::entry_split(
                &mut *graph,
                entry,
                width,
            );
        }
        SideIndexable::<_, D, IndexFront>::entry_split(
            &mut *graph,
            entry,
            width,
        )
    }
    fn index_mismatch<Acc, Q: TraversalQuery + ReduciblePath>(
        &'a mut self,
        acc: Acc,
        paths: PathPair<Q, SearchPath>,
    ) -> ControlFlow<(Child, Q), Acc> {
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
    fn index_range_path(
        &'a mut self,
        path: SearchPath,
    ) -> Child {
        let start_width = path.start.width();
        let inner_width = path.inner_width;
        let end_width = path.end.width();
        let entry_pos = path.start.get_entry_pos();
        let exit_pos = path.end.get_exit_pos();
        let root@PatternLocation {
            parent,
            ..
        } = path.start.entry().into_pattern_location();
        let mut graph = self.graph_mut();
        let pattern = path.start.pattern(&*graph);

        let IndexSplitResult {
            inner: prefix,
            path: prefix_path,
            location: prefix_location,
        } = SideIndexable::<_, D, IndexBack>::pattern_entry_split(
            &mut *graph,
            pattern.borrow(),
            path.start.entry(),
            start_width,
        );
        // todo: index inner

        let IndexSplitResult {
            inner: postfix,
            path: postfix_path,
            location: postfix_location,
        } = SideIndexable::<_, D, IndexFront>::pattern_entry_split(
            &mut *graph,
            pattern.borrow(),
            path.end.entry(),
            end_width,
        );
        // todo: create index
        end_split.inner
    }
    fn pattern_range_perfect_split(
        &'a mut self,
        pattern: impl IntoPattern,
        location: impl IntoPatternLocation,
        inner_range: impl PatternRangeIndex + StartInclusive,
    ) -> IndexSplitResult {
        let pos = inner_range.start();
        let inner = &pattern.borrow()[inner_range.clone()];
        let location = location.into_pattern_location().to_child_location(pos);
        let inner = if inner.len() == 1 {
            *inner.iter().next().unwrap()
        } else {
            assert!(inner.len() > 0);
            let mut graph = self.graph_mut();
            let inner = graph.index_pattern(inner);
            graph.replace_in_pattern(&location, inner_range, [inner]);
            inner
        };
        IndexSplitResult {
            location,
            path: vec![location],
            inner,
        }
    }
    //fn range_both_offset_split(
    //    &'a mut self,
    //    pattern: Pattern,
    //    root: impl IntoPatternLocation,
    //    start: usize,
    //    start_offset: NonZeroUsize,
    //    end: usize,
    //    end_offset: NonZeroUsize,
    //) -> IndexSplitResult {
    //    let root = root.into_pattern_location();
    //    let child_patterns = self.graph().expect_children_of(root.parent).clone();
    //    let pattern = root.expect_pattern_in(&child_patterns);
    //    let pre_width = pattern[..start].width();
    //    let back_index = &pattern[start];
    //    // pre_width + <IndexBack as IndexSide<D>>::width_offset(back_index, start_width);
    //    let front_index = &pattern[end];
    //    //pre_width + back_index.width() + inner_width + <IndexFront as IndexSide<D>>::width_offset(front_index, end_width);
    //    let positions = child_patterns.into_iter()
    //        .map(|(pid, pattern)| {
    //            let (back_index, back_offset) = <IndexBack as IndexSide<D>>::token_offset_split(pattern.borrow(), start_offset).unwrap();
    //            let (front_index, front_offset) = <IndexFront as IndexSide<D>>::token_offset_split(pattern.borrow(), end_offset).unwrap();
    //            (pid, pattern.into_pattern(), back_index, back_offset, front_index, front_offset)
    //        })
    //        .collect_vec();
    //    let (backs, inners, fronts): (Vec<_>, Vec<_>, Vec<_>) = multiunzip(positions.into_iter()
    //        .map(|(_, pattern, back_pos, back_offset, front_pos, front_offset)| {
    //            let IndexSplitResult {
    //                inner: back_inner,
    //                context: back_context_path,
    //                location: back_location,
    //            } = SideIndexable::<_, D, IndexBack>::index_offset_split(
    //                self,
    //                *pattern.get(back_pos).unwrap(),
    //                back_offset,
    //            );
    //            let back_context = SideIndexable::<_, D, IndexBack>::index_context_path(
    //                self,
    //                back_location,
    //                back_context_path,
    //            );

    //            let IndexSplitResult {
    //                inner: front_inner,
    //                context: front_context_path,
    //                location: front_location,
    //            } = SideIndexable::<_, D, IndexFront>::index_offset_split(
    //                self,
    //                *pattern.get(front_pos).unwrap(),
    //                front_offset,
    //            );
    //            let front_context = SideIndexable::<_, D, IndexFront>::index_context_path(
    //                self,
    //                front_location,
    //                front_context_path,
    //            );

    //            let inner_context = pattern
    //                .get(D::inner_context_range(back_pos, front_pos))
    //                .and_then(|p| self.insert_pattern(p));
    //            let inner_context = inner_context.as_ref()
    //                .map(std::slice::from_ref)
    //                .unwrap_or_default();
    //            (
    //                // todo: order depends on D
    //                [&D::back_context(pattern.borrow(), back_pos)[..], &[back_context]].concat(),
    //                [&[back_inner], inner_context, &[front_inner]].concat(),
    //                [&[front_context], &D::front_context(pattern.borrow(), front_pos)[..]].concat(),
    //            )
    //        }));
    //    let graph = self.graph_mut();
    //    let (back, inner, front) = (
    //        graph.index_patterns(backs),
    //        graph.index_patterns(inners),
    //        graph.index_patterns(fronts),
    //    );
    //    let pid = self.add_pattern_with_update(parent, [back, inner, front]);
    //    let location = ChildLocation::new(parent, pid, 1);
    //    IndexSplitResult {
    //        location,
    //        context: vec![],
    //        inner,
    //    }
    //}
    //fn range_perfect_split(
    //    &'a mut self,
    //    root: impl IntoPatternLocation,
    //    range: impl PatternRangeIndex + StartInclusive,
    //) -> IndexSplitResult {
    //    let root = root.into_pattern_location();
    //    let pattern = root.expect_pattern(self);
    //    self.pattern_range_perfect_split(pattern, root, range)
    //}
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