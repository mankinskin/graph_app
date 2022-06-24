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
            path.start.width()
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
    //fn index_end_path(
    //    &'a mut self,
    //    end: EndPath,
    //) -> IndexSplitResult {
    //    let EndPath {
    //        mut path,
    //        entry,
    //        width,
    //    } = end;
    //    let mut graph = self.graph_mut();
    //    while let Some(entry) = path.pop() {
    //        SideIndexable::<_, D, IndexFront>::entry_split(
    //            &mut *graph,
    //            entry,
    //            width,
    //        );
    //    }
    //    SideIndexable::<_, D, IndexFront>::entry_split(
    //        &mut *graph,
    //        entry,
    //        width,
    //    )
    //}
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
        let end_width = path.end.width();
        let entry_pos = path.start.get_entry_pos();
        let exit_pos = path.end.get_exit_pos();
        let location = path.start.entry().into_pattern_location();

        let mut graph = self.graph_mut();

        let range = D::wrapper_range(entry_pos, exit_pos);
        assert!(range.end() - range.start() > 0, "No more than a single index in range path");

        let wrapper = graph.index_range_in(
            location,
            range,
        ).unwrap();

        let (pid, pattern) = wrapper.patterns(&*graph).into_iter().next().unwrap();
        let location = wrapper.to_pattern_location(pid);

        let head_pos = D::head_index(pattern.borrow());
        let last_pos = D::last_index(pattern.borrow());

        let head = pattern[head_pos];
        let last = pattern[last_pos];

        let head_split = <IndexBack as IndexSide<D>>::inner_width_to_offset(&head, start_width)
            .map(|offset| {
                let head_split = SideIndexable::<_, D, IndexBack>::single_offset_split(
                    &mut *graph,
                    head,
                    offset,
                );
                let head_context = SideIndexable::<_, D, IndexBack>::context_path(
                    &mut *graph,
                    head_split.location,
                    head_split.path,
                );
                (head_split.inner, head_context)
            });
        let last_split = <IndexFront as IndexSide<D>>::inner_width_to_offset(&last, end_width).map(|offset| {
            let last_split = SideIndexable::<_, D, IndexFront>::single_offset_split(
                &mut *graph,
                last,
                offset,
            );
            let last_context = SideIndexable::<_, D, IndexFront>::context_path(
                &mut *graph,
                last_split.location,
                last_split.path,
            );
            (last_split.inner, last_context)
        });
        match (head_split, last_split) {
            (Some((head_inner, head_context)), Some((last_inner, last_context))) => {
                let inner = graph.index_range_in(
                    location,
                    D::inner_context_range(head_pos, last_pos),
                ).ok();
                let target = graph.insert_pattern(
                    D::concat_context_inner_context(
                        head_inner,
                        inner.as_ref().map(std::slice::from_ref).unwrap_or_default(),
                        last_inner
                    )
                ).unwrap();
                graph.add_pattern_with_update(
                    wrapper,
                    D::concat_context_inner_context(head_context, target, last_context)
                );
                target
            },
            (Some((head_inner, head_context)), None) => {
                let inner = graph.index_range_in(
                    location,
                    <IndexBack as IndexSide<D>>::inner_range(head_pos),
                ).unwrap();
                let target = graph.insert_pattern(
                    D::concat_inner_and_context(head_inner, inner)
                ).unwrap();
                graph.add_pattern_with_update(
                    wrapper,
                    D::concat_context_and_inner(head_context, target)
                );
                target
            },
            (None, Some((last_inner, last_context))) => {
                let inner = graph.index_range_in(
                    location,
                    <IndexFront as IndexSide<D>>::inner_range(last_pos)
                ).unwrap();
                let target = graph.insert_pattern(
                    D::concat_context_and_inner(last_inner, inner)
                ).unwrap();
                graph.add_pattern_with_update(
                    wrapper,
                    D::concat_inner_and_context(target, last_context)
                );
                target
            },
            (None, None) => wrapper,
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