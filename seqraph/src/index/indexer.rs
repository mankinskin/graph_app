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
    ) -> MatchEnd {
        let mut ltrav = trav.clone();
        let entry = path.get_entry_location();
        if let Some(IndexSplitResult {
            inner: post,
            location: entry,
            ..
            // should call leaf split and use known info of leaf position
        }) = SideIndexable::<_, D, IndexBack>::entry_perfect_split(
            &mut ltrav,
            entry,
        ) {
            MatchEnd::Path(StartPath::Leaf(StartLeaf { entry, child: post, width: path.width() }))
        } else {
            MatchEnd::Complete(entry.parent)
        }
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
    type Node = IndexingNode<Q>;
    fn fold_found(
        trav: &Self::Trav,
        acc: Self::Continue,
        node: Self::Node,
    ) -> ControlFlow<Self::Break, Self::Continue> {
        let mut trav = trav.clone();
        match node {
            IndexingNode::QueryEnd(Some(found)) => {
                ControlFlow::Break((
                    Indexable::<_, D>::index_found(&mut trav, found.found),
                    found.query,
                ))
            },
            IndexingNode::Mismatch(paths) => {
                let (path, query) = paths.unpack();
                let found = FoundPath::new::<_, D, _>(&trav, path);
                ControlFlow::Break((
                    Indexable::<_, D>::index_found(&mut trav, found),
                    query
                ))
            },
            IndexingNode::Match(path, query) =>
                ControlFlow::Continue(search::fold_match::<_, _, _, Self>(&trav, acc, path, query)),
            IndexingNode::MatchEnd(match_end, query) => {
                let found = TraversalResult::new(
                    match_end,
                    query,
                );
                ControlFlow::Continue(Some(found))
            },
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
    
    fn index_range_path(
        &'a mut self,
        path: SearchPath,
    ) -> Child {
        let entry_pos = path.start.get_entry_pos();
        let exit_pos = path.end.get_exit_pos();
        let location = path.start.entry();
        let mut graph = self.graph_mut();

        // a little bit dirty, path should have typing for this
        if entry_pos == exit_pos && path.start.path().is_empty() && path.end.path().is_empty() {
            return graph.expect_child_at(&location);
        }
        let location = location.into_pattern_location();

        let range = D::wrapper_range(entry_pos, exit_pos);
        graph.validate_pattern_indexing_range_at(&location, entry_pos, exit_pos).unwrap();
        let (wrapper, pattern, location) = if let Ok(wrapper) =
            graph.index_range_in(
                location,
                range,
            ) {
                let (pid, pattern) = wrapper.expect_child_patterns(&*graph).into_iter().next().unwrap();
                let location = wrapper.to_pattern_location(pid);
                (wrapper, pattern, location)
            } else {
                let wrapper = location.parent;
                let pattern = wrapper.expect_child_pattern(&*graph, location.pattern_id);
                (wrapper, pattern, location)
            };

        let head_pos = D::head_index(pattern.borrow());
        let last_pos = D::last_index(pattern.borrow());

        let head_split = SideIndexable::<_, D, IndexBack>::single_path_split(
            &mut *graph,
            path.start.path().to_vec()
        ).map(|split| (
            split.inner,
            SideIndexable::<_, D, IndexBack>::context_path(
                &mut *graph,
                split.location,
                split.path,
            )
        ));
        let last_split =
            SideIndexable::<_, D, IndexFront>::single_path_split(
                &mut *graph,
                path.end.path().to_vec()
            ).map(|split| (
                split.inner,
                SideIndexable::<_, D, IndexFront>::context_path(
                    &mut *graph,
                    split.location,
                    split.path,
                )
            ));

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
                let inner_context = graph.index_range_in_or_default(
                    location,
                    <IndexBack as IndexSide<D>>::inner_context_range(head_pos),
                ).unwrap();
                let target = graph.insert_pattern(
                    D::concat_inner_and_context(head_inner, inner_context)
                ).unwrap();
                graph.add_pattern_with_update(
                    wrapper,
                    D::concat_context_and_inner(head_context, target)
                );
                target
            },
            (None, Some((last_inner, last_context))) => {
                let inner_context = graph.index_range_in_or_default(
                    location,
                    <IndexFront as IndexSide<D>>::inner_context_range(last_pos)
                ).unwrap();
                let target = graph.insert_pattern(
                    D::concat_context_and_inner(last_inner, inner_context)
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
    pub(crate) fn index_pattern(
        &mut self,
        query: impl IntoPattern,
    ) -> Result<(Child, QueryRangePath), NoMatch> {
        let query = query.into_pattern();
        let query_path = QueryRangePath::new_directed::<D, _>(query.borrow())?;
        self.index_query(query_path)
    }
    pub(crate) fn index_query<
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
        let mut visited = Vec::new();
        match Ti::new(self, TraversalNode::query_node(query_path))
            .try_fold(
                None,
                |acc, (depth, node)| {
                    if visited.iter().find(|(_, n)| *n == node).is_some() {
                        panic!("cycle detected");
                        //ControlFlow::Continue(acc)
                    } else {
                        visited.push((depth, node.clone()));
                        S::Folder::fold_found(self, acc, node)
                    }
                }
            )
        {
            ControlFlow::Continue(None) => Err(NoMatch::NotFound),
            ControlFlow::Continue(Some(found)) => Ok((Indexable::<_, D>::index_found(&mut self.clone(), found.found), found.query)),
            ControlFlow::Break((found, query)) => Ok((found, query))
        }
    }
}