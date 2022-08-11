use crate::{
    search::*,
    Hypergraph,
};
use std::{sync::RwLockReadGuard, ops::ControlFlow};
use rayon::iter::*;

#[derive(Clone, Debug)]
pub struct Searcher<T: Tokenize, D: MatchDirection> {
    graph: HypergraphRef<T>,
    _ty: std::marker::PhantomData<D>,
}
impl<'a: 'g, 'g, T: Tokenize + 'a, D: MatchDirection> Traversable<'a, 'g, T> for Searcher<T, D> {
    type Guard = RwLockReadGuard<'g, Hypergraph<T>>;
    fn graph(&'g self) -> Self::Guard {
        self.graph.read().unwrap()
    }
}

trait SearchTraversalPolicy<
    'a: 'g,
    'g,
    T: Tokenize + 'a,
    D: MatchDirection + 'a,
>:
    DirectedTraversalPolicy<'a, 'g, T, D, QueryRangePath, Trav=Searcher<T, D>, Folder=Searcher<T, D>>
{
}
impl<
    'a: 'g,
    'g,
    T: Tokenize + 'a,
    D: MatchDirection + 'a,
>
    SearchTraversalPolicy<'a, 'g, T, D> for AncestorSearch<T, D>
{}
impl<
    'a: 'g,
    'g,
    T: Tokenize + 'a,
    D: MatchDirection + 'a,
>
    SearchTraversalPolicy<'a, 'g, T, D> for ParentSearch<T, D>
{}

impl<'a: 'g, 'g, T: Tokenize + 'a, D: MatchDirection>
    TraversalFolder<'a, 'g, T, D, QueryRangePath> for Searcher<T, D>
{
    type Trav = Self;
    type Break = Option<QueryFound>;
    type Continue = Option<QueryFound>;
    type Path = SearchPath;
    fn fold_found(
        trav: &Self::Trav,
        acc: Self::Continue,
        node: MatchNode,
    ) -> ControlFlow<Self::Break, Self::Continue> {
        match node {
            MatchNode::QueryEnd(found) => {
                ControlFlow::Break(found)
            },
            MatchNode::Match(path, query) =>
                ControlFlow::Continue(fold_match::<_, _, _, Self>(trav, acc, path, query)),
            _ => ControlFlow::Continue(acc)
        }
    }
}
pub(crate) fn fold_match<
    'a: 'g,
    'g,
    T: Tokenize + 'a,
    D: MatchDirection + 'a,
    Q: TraversalQuery,
    Folder: TraversalFolder<'a, 'g, T, D, Q, Continue=Option<TraversalResult<SearchPath, Q>>>
>(
    trav: &'a Folder::Trav,
    acc: Folder::Continue,
    path: SearchPath,
    query: Q
) -> Option<TraversalResult<SearchPath, Q>> {
    let found = TraversalResult::new(
        path.reduce_end::<_, D, _>(trav),
        query
    );
    Some(
        if let Some(acc) = acc {
            std::cmp::max_by(
                found,
                acc,
                |found, acc|
                    found.found.cmp(&acc.found)
            )
        } else {
            found
        }
    )
}
struct AncestorSearch<T: Tokenize, D: MatchDirection> {
    _ty: std::marker::PhantomData<(T, D)>,
}
impl<
    'a: 'g,
    'g,
    T: Tokenize + 'a,
    D: MatchDirection + 'a,
>
    DirectedTraversalPolicy<'a, 'g, T, D, QueryRangePath> for AncestorSearch<T, D>
{
    type Trav = Searcher<T, D>;
    type Folder = Searcher<T, D>;
}
struct ParentSearch<T: Tokenize, D: MatchDirection> {
    _ty: std::marker::PhantomData<(T, D)>,
}
impl<
    'a: 'g,
    'g,
    T: Tokenize + 'a,
    D: MatchDirection + 'a,
>
    DirectedTraversalPolicy<'a, 'g, T, D, QueryRangePath> for ParentSearch<T, D>
{
    type Trav = Searcher<T, D>;
    type Folder = Searcher<T, D>;
    fn next_parents(
        _trav: &'a Self::Trav,
        _query: &QueryRangePath,
        _start: &MatchEnd,
    ) -> Vec<MatchNode> {
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
        S: SearchTraversalPolicy<'a, 'g, T, D, > + Send,
        P: IntoPattern,
    >(
        &'a self,
        query: P,
    ) -> SearchResult {
        self.par_search::<Dft<'a, 'g, T, D, Self, QueryRangePath, S>, S, _>(
            query,
        )
    }
    fn search<
        Ti: TraversalIterator<'a, 'g, T, D, Self, QueryRangePath, S> + Send,
        S: SearchTraversalPolicy<'a, 'g, T, D>,
        P: IntoPattern,
    >(
        &'a self,
        query: P,
    ) -> SearchResult {
        let query_path = QueryRangePath::new_directed::<D, _>(query.borrow())?;
        match Ti::new(self, TraversalNode::query_node(query_path))
            .try_fold(None, |acc, (_, node)|
                S::Folder::fold_found(self, acc, node)
            )
        {
            ControlFlow::Continue(None) => Err(NoMatch::NotFound),
            ControlFlow::Continue(Some(found)) => Ok(found),
            ControlFlow::Break(found) => found.ok_or(NoMatch::SingleIndex)
        }
    }
    #[allow(unused)]
    fn par_search<
        Ti: TraversalIterator<'a, 'g, T, D, Self, QueryRangePath, S> + Send,
        S: SearchTraversalPolicy<'a, 'g, T, D>,
        P: IntoPattern,
    >(
        &'a self,
        query: P,
    ) -> SearchResult {
        let query_path = QueryRangePath::new_directed::<D, _>(query.borrow())?;
        match Ti::new(self, TraversalNode::query_node(query_path))
            .par_bridge()
            .try_fold_with(None, |acc, (_, node)|
                S::Folder::fold_found(self, acc, node)
            )
            .reduce(|| ControlFlow::Continue(None), |a, b|
                match (a, b) {
                    (ControlFlow::Break(b), ControlFlow::Continue(_)) |
                    (ControlFlow::Continue(_), ControlFlow::Break(b)) =>
                        ControlFlow::Break(b),
                    (ControlFlow::Break(a), ControlFlow::Break(b)) =>
                        ControlFlow::Break(match (a, b) {
                            (None, None) => None,
                            (None, Some(found)) |
                            (Some(found), None) => Some(found),
                            (Some(a), Some(b)) =>
                                Some(std::cmp::max_by(a, b, |a, b| a.found.cmp(&b.found)))
                        }),
                    (ControlFlow::Continue(a), ControlFlow::Continue(b)) =>
                        ControlFlow::Continue(match (a, b) {
                            (None, None) => None,
                            (None, Some(found)) |
                            (Some(found), None) => Some(found),
                            (Some(a), Some(b)) =>
                                Some(std::cmp::max_by(a, b, |a, b| a.found.cmp(&b.found)))
                        })
                }
            )
        {
            ControlFlow::Continue(None) => Err(NoMatch::NotFound),
            ControlFlow::Continue(Some(found)) => Ok(found),
            ControlFlow::Break(found) => found.ok_or(NoMatch::SingleIndex)
        }
    }
}
