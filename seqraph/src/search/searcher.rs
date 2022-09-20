use crate::{
    search::*,
    Hypergraph,
};
use std::{sync::RwLockReadGuard, ops::ControlFlow};
use rayon::iter::{
    ParallelBridge,
    ParallelIterator,
};

#[derive(Clone, Debug)]
pub struct Searcher<T: Tokenize, D: MatchDirection> {
    graph: HypergraphRef<T>,
    _ty: std::marker::PhantomData<D>,
}
impl<'a: 'g, 'g, T: Tokenize + 'a, D: MatchDirection + 'a> Traversable<'a, 'g, T> for Searcher<T, D> {
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
    DirectedTraversalPolicy<
        'a, 'g, T, D,
        QueryRangePath,
        MatchEndResult,
        Trav=Searcher<T, D>,
        Folder=Searcher<T, D>,
        AfterEndMatch=<MatchEndResult as ResultKind>::Result<StartPath>,
    >
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

impl<'a: 'g, 'g, T: Tokenize + 'a, D: MatchDirection + 'a, Q: TraversalQuery, R: ResultKind>
    TraversalFolder<'a, 'g, T, D, Q, R> for Searcher<T, D>
{
    type Trav = Self;
    type Break = TraversalResult<Q>;
    type Continue = Option<TraversalResult<Q>>;
    type AfterEndMatch = R::Result<StartPath>;
    fn fold_found(
        _trav: &Self::Trav,
        acc: Self::Continue,
        node: TraversalNode<R::Result<StartPath>, Q>,
    ) -> ControlFlow<Self::Break, Self::Continue> {
        match node {
            TraversalNode::QueryEnd(found) => {
                ControlFlow::Break(found)
            },
            TraversalNode::MatchEnd(match_end, query) => {
                let found = TraversalResult::new(
                    FoundPath::from(match_end.into_mesp()),
                    query,
                );
                ControlFlow::Continue(search::pick_max_result(acc, found))
            },
            TraversalNode::Mismatch(found) =>
                ControlFlow::Continue(search::pick_max_result(acc, found)),
            _ => ControlFlow::Continue(acc)
        }
    }
}
pub(crate) fn pick_max_result<
    Q: TraversalQuery,
>(
    acc: Option<TraversalResult<Q>>,
    res: TraversalResult<Q>,
) -> Option<TraversalResult<Q>> {
    Some(
        if let Some(acc) = acc {
            std::cmp::max_by(
                res,
                acc,
                |res, acc|
                    res.found.cmp(&acc.found)
            )
        } else {
            res
        }
    )
}
pub(crate) fn fold_match<
    'a: 'g,
    'g,
    T: Tokenize + 'a,
    D: MatchDirection + 'a,
    Q: TraversalQuery,
    Folder: TraversalFolder<'a, 'g, T, D, Q, MatchEndResult, Continue=Option<TraversalResult<Q>>>
>(
    trav: &'a Folder::Trav,
    acc: Folder::Continue,
    mut path: SearchPath,
    query: Q
) -> Option<TraversalResult<Q>> {
    path.reduce_end::<_, D, _>(trav);
    let found = TraversalResult::new(
        FoundPath::new::<_, D, _>(trav, path),
        query
    );
    pick_max_result(acc, found)
}
struct AncestorSearch<T: Tokenize, D: MatchDirection> {
    _ty: std::marker::PhantomData<(T, D)>,
}
impl<
    'a: 'g,
    'g,
    T: Tokenize + 'a,
    D: MatchDirection + 'a,
    R: ResultKind,
>
    DirectedTraversalPolicy<'a, 'g, T, D, QueryRangePath, R> for AncestorSearch<T, D>
{
    type Trav = Searcher<T, D>;
    type Folder = Searcher<T, D>;
    type AfterEndMatch = R::Result<StartPath>;

    fn after_end_match(
        _trav: &'a Self::Trav,
        path: StartPath,
    ) -> Self::AfterEndMatch {
        Self::AfterEndMatch::from(path)
    }
}
struct ParentSearch<T: Tokenize, D: MatchDirection> {
    _ty: std::marker::PhantomData<(T, D)>,
}
impl<
    'a: 'g,
    'g,
    T: Tokenize + 'a,
    D: MatchDirection + 'a,
    Q: TraversalQuery,
    R: ResultKind,
>
    DirectedTraversalPolicy<'a, 'g, T, D, Q, R> for ParentSearch<T, D>
{
    type Trav = Searcher<T, D>;
    type Folder = Searcher<T, D>;
    type AfterEndMatch = R::Result<StartPath>;
    fn after_end_match(
        _trav: &'a Self::Trav,
        path: StartPath,
    ) -> Self::AfterEndMatch {
        Self::AfterEndMatch::from(path)
    }
    fn next_parents(
        _trav: &'a Self::Trav,
        _query: &Q,
        _start: &MatchEnd<StartPath>,
    ) -> Vec<TraversalNode<Self::AfterEndMatch, Q>> {
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
        S: SearchTraversalPolicy<'a, 'g, T, D> + Send,
        P: IntoPattern,
    >(
        &'a self,
        query: P,
    ) -> SearchResult {
        self.search::<Dft<'a, 'g, T, D, Self, QueryRangePath, MatchEndResult, S>, S, _>(
            query,
        )
    }
    #[allow(unused)]
    fn search<
        Ti: TraversalIterator<'a, 'g, T, D, Self, QueryRangePath, S, MatchEndResult> + Send,
        S: SearchTraversalPolicy<'a, 'g, T, D>,
        P: IntoPattern,
    >(
        &'a self,
        query: P,
    ) -> SearchResult {
        let query_path = QueryRangePath::new_directed::<D, _>(query.borrow())?;
        match Ti::new(self, TraversalNode::query_node(query_path))
            .try_fold(None, |acc, (_, node)|
                <S::Folder as TraversalFolder<_, _, _, MatchEndResult>>::fold_found(self, acc, node)
            )
        {
            ControlFlow::Continue(None) => Err(NoMatch::NotFound),
            ControlFlow::Continue(Some(found)) |
            ControlFlow::Break(found) => Ok(found)
        }
    }
    #[allow(unused)]
    fn par_search<
        Ti: TraversalIterator<'a, 'g, T, D, Self, QueryRangePath, S, MatchEndResult> + Send,
        S: SearchTraversalPolicy<'a, 'g, T, D>,
        P: IntoPattern,
    >(
        &'a self,
        query: P,
    ) -> SearchResult {
        let query_path = QueryRangePath::new_directed::<D, _>(query.borrow())?;
        match ParallelIterator::reduce(
            Ti::new(self, TraversalNode::query_node(query_path))
                .par_bridge()
                .try_fold_with(None, |acc, (_, node)|
                    <S::Folder as TraversalFolder<_, _, _, MatchEndResult>>::fold_found(self, acc, node)
                ),
                || ControlFlow::Continue(None), |a, b|
                    match (a, b) {
                        (ControlFlow::Break(b), ControlFlow::Continue(_)) |
                        (ControlFlow::Continue(_), ControlFlow::Break(b)) =>
                            ControlFlow::Break(b),
                        (ControlFlow::Break(a), ControlFlow::Break(b)) =>
                            ControlFlow::Break(
                                std::cmp::max_by(
                                    a,
                                    b,
                                    |a, b|
                                        a.found.cmp(&b.found)
                                )
                            ),
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
            ControlFlow::Continue(Some(found)) |
            ControlFlow::Break(found) => Ok(found)
        }
    }
}
