use crate::{
    search::*,
    Hypergraph,
};
use std::{sync::RwLockReadGuard, ops::ControlFlow, borrow::Borrow};

#[derive(Clone)]
pub struct Searcher<T: Tokenize, D: MatchDirection> {
    graph: HypergraphRef<T>,
    _ty: std::marker::PhantomData<D>,
}
impl<'a: 'g, 'g, T: Tokenize + 'a, D: MatchDirection + 'a> Traversable<'a, 'g, T> for Searcher<T, D> {
    type Guard = RwLockReadGuard<'g, Hypergraph<T>>;
    fn graph(&'a self) -> Self::Guard {
        self.graph.read().unwrap()
    }
}

trait SearchTraversalPolicy<'a: 'g, 'g, T: Tokenize + 'a, D: MatchDirection + 'a>:
    DirectedTraversalPolicy<'a, 'g, T, D, Trav=Searcher<T, D>, Folder=Searcher<T, D>>
{}
impl<'a: 'g, 'g, T: Tokenize + 'a, D: MatchDirection + 'a> SearchTraversalPolicy<'a, 'g, T, D> for AncestorSearch<T, D> {}
impl<'a: 'g, 'g, T: Tokenize + 'a, D: MatchDirection + 'a> SearchTraversalPolicy<'a, 'g, T, D> for ParentSearch<T, D> {}

impl<'a: 'g, 'g, T: Tokenize + 'a, D: MatchDirection + 'a> TraversalFolder<'a, 'g, T, D> for Searcher<T, D> {
    type Trav = Self;
    type Break = Option<QueryFound>;
    type Continue = Option<QueryFound>;
    fn fold_found(
        trav: &Self::Trav,
        acc: Self::Continue,
        node: TraversalNode,
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
struct AncestorSearch<T: Tokenize, D: MatchDirection> {
    _ty: std::marker::PhantomData<(T, D)>,
}
impl<'a: 'g, 'g, T: Tokenize + 'a, D: MatchDirection + 'a> DirectedTraversalPolicy<'a, 'g, T, D> for AncestorSearch<T, D> {
    type Trav = Searcher<T, D>;
    type Folder = Searcher<T, D>;
    fn end_op(
        trav: &'a Self::Trav,
        query: QueryRangePath,
        start: StartPath,
    ) -> Vec<TraversalNode> {
        Self::parent_nodes(trav, query, Some(start))
    }
}
struct ParentSearch<T: Tokenize, D: MatchDirection> {
    _ty: std::marker::PhantomData<(T, D)>,
}
impl<'a: 'g, 'g, T: Tokenize + 'a, D: MatchDirection + 'a> DirectedTraversalPolicy<'a, 'g, T, D> for ParentSearch<T, D> {
    type Trav = Searcher<T, D>;
    type Folder = Searcher<T, D>;
    fn end_op(
        _trav: &'a Self::Trav,
        _query: QueryRangePath,
        _start: StartPath,
    ) -> Vec<TraversalNode> {
        vec![]
    }
}
impl<'a: 'g, 'g, T: Tokenize + 'g, D: MatchDirection + 'g> Searcher<T, D> {
    pub fn new(graph: HypergraphRef<T>) -> Self {
        Self {
            graph,
            _ty: Default::default(),
        }
    }
    // find largest matching direct parent
    pub(crate) fn find_pattern_parent(
        &'a self,
        pattern: impl IntoPattern,
    ) -> SearchResult {
        self.dft_search::<ParentSearch<T, D>, _>(
            pattern,
        )
    }
    /// find largest matching ancestor for pattern
    pub(crate) fn find_pattern_ancestor(
        &'a self,
        pattern: impl IntoPattern,
    ) -> SearchResult {
        self.dft_search::<AncestorSearch<T, D>, _>(
            pattern,
        )
    }
    fn dft_search<
        S: SearchTraversalPolicy<'a, 'g, T, D>,
        Q: IntoPattern,
    >(
        &'a self,
        query: Q,
    ) -> SearchResult {
        self.search::<Dft<_, _, _, _>, S, _>(
            query,
        )
    }
    #[allow(unused)]
    fn bft_search<
        S: SearchTraversalPolicy<'a, 'g, T, D>,
        Q: IntoPattern,
    >(
        &'a self,
        query: Q,
    ) -> SearchResult {
        self.search::<Bft<_, _, _, _>, S, _>(
            query,
        )
    }
    fn search<
        Ti: TraversalIterator<'a, 'g, T, Self, D, S>,
        S: SearchTraversalPolicy<'a, 'g, T, D>,
        Q: IntoPattern,
    >(
        &'a self,
        query: Q,
    ) -> SearchResult {
        let query = query.into_pattern();
        let query_path = QueryRangePath::new_directed::<D, _>(query.borrow())?;
        match Ti::new(self, TraversalNode::Query(query_path))
            .try_fold(None, |acc: Option<QueryFound>, (_, node)|
                S::Folder::fold_found(self, acc, node)
            )
        {
            ControlFlow::Continue(None) => Err(NoMatch::NotFound(query)),
            ControlFlow::Continue(Some(found)) => Ok(found),
            ControlFlow::Break(found) => found.ok_or(NoMatch::SingleIndex)
        }
    }
}
