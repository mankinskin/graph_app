use crate::{
    search::*,
    Hypergraph,
};
use std::ops::ControlFlow;
//use rayon::iter::{
//    ParallelBridge,
//    ParallelIterator,
//};

#[derive(Clone, Debug)]
pub struct Searcher<T: Tokenize, D: MatchDirection> {
    graph: HypergraphRef<T>,
    _ty: std::marker::PhantomData<D>,
}

impl<T: Tokenize, D: MatchDirection> Traversable<T> for Searcher<T, D> {
    type Guard<'g> = RwLockReadGuard<'g, Hypergraph<T>> where Self: 'g;
    fn graph<'g>(&'g self) -> Self::Guard<'g> {
        self.graph.read().unwrap()
    }
}

impl<T: Tokenize, D: MatchDirection> Traversable<T> for &'_ Searcher<T, D> {
    type Guard<'g> = RwLockReadGuard<'g, Hypergraph<T>> where Self: 'g;
    fn graph<'g>(&'g self) -> Self::Guard<'g> {
        self.graph.read().unwrap()
    }
}

trait SearchTraversalPolicy<
    T: Tokenize,
    D: MatchDirection,
>:
    DirectedTraversalPolicy<
        T, D,
        QueryRangePath,
        BaseResult,
        Trav=Searcher<T, D>,
        Folder=Searcher<T, D>,
        //Primer=StartPath,
    >
{
}
impl<
    T: Tokenize,
    D: MatchDirection,
>
    SearchTraversalPolicy<T, D> for AncestorSearch<T, D>
{}
impl<
    T: Tokenize,
    D: MatchDirection,
>
    SearchTraversalPolicy<T, D> for ParentSearch<T, D>
{}


impl<T: Tokenize, D: MatchDirection, Q: TraversalQuery, R: ResultKind>
    TraversalFolder<T, D, Q, R> for Searcher<T, D>
{
    type Trav = Self;
    type Break = TraversalResult<<R as ResultKind>::Found, Q>;
    type Continue = Option<TraversalResult<<R as ResultKind>::Found, Q>>;

    fn fold_found(
        _trav: &Self::Trav,
        acc: Self::Continue,
        node: TraversalNode<R, Q>,
    ) -> ControlFlow<Self::Break, Self::Continue> {
        match node {
            TraversalNode::QueryEnd(found) => {
                ControlFlow::Break(found)
            },
            TraversalNode::MatchEnd(match_end, query) => {
                let found = TraversalResult::new(
                    match_end,
                    query,
                );
                ControlFlow::Continue(search::pick_max_result::<R, _>(acc, found))
            },
            TraversalNode::Mismatch(found) =>
                ControlFlow::Continue(search::pick_max_result::<R, _>(acc, found)),
            _ => ControlFlow::Continue(acc)
        }
    }
}
pub(crate) fn pick_max_result<
    R: ResultKind,
    Q: TraversalQuery,
>(
    acc: Option<TraversalResult<<R as ResultKind>::Found, Q>>,
    res: TraversalResult<<R as ResultKind>::Found, Q>,
) -> Option<TraversalResult<<R as ResultKind>::Found, Q>> {
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
//pub(crate) fn fold_match<
//    'a: 'g,
//    'g,
//    T: Tokenize,
//    D: MatchDirection,
//    Q: TraversalQuery,
//    R: ResultKind,
//    Folder: TraversalFolder<T, D, Q, R, Continue=Option<TraversalResult<<R as ResultKind>::Found, Q>>>
//>(
//    trav: &'a Folder::Trav,
//    acc: Folder::Continue,
//    mut path: SearchPath,
//    query: Q
//) -> Option<TraversalResult<<R as ResultKind>::Found, Q>> {
//    path.end_match_path_mut().reduce::<_, D, _>(trav);
//    //let found = 
//    //    path.into_range_path().into_result(query);
//    pick_max_result(acc, path)
//}
struct AncestorSearch<T: Tokenize, D: MatchDirection> {
    _ty: std::marker::PhantomData<(T, D)>,
}

impl<
    T: Tokenize,
    D: MatchDirection,
    R: ResultKind,
>
    DirectedTraversalPolicy<T, D, QueryRangePath, R> for AncestorSearch<T, D>
{
    type Trav = Searcher<T, D>;
    type Folder = Searcher<T, D>;
    //type Primer = R::Result<StartPath>;

    fn after_end_match(
        _trav: &Self::Trav,
        path: R::Primer,
    ) -> R::Postfix {
        R::Postfix::from(path)
    }
}
struct ParentSearch<T: Tokenize, D: MatchDirection> {
    _ty: std::marker::PhantomData<(T, D)>,
}

impl<
    'a: 'g,
    'g,
    T: Tokenize,
    D: MatchDirection,
    Q: TraversalQuery,
    R: ResultKind,
>
    DirectedTraversalPolicy<T, D, Q, R> for ParentSearch<T, D>
{
    type Trav = Searcher<T, D>;
    type Folder = Searcher<T, D>;

    fn after_end_match(
        _trav: &Self::Trav,
        path: R::Primer,
    ) -> R::Postfix {
        R::Postfix::from(path)
    }
    fn next_parents(
        _trav: &Self::Trav,
        _query: &Q,
        _start: &R::Postfix,
    ) -> Vec<TraversalNode<R, Q>> {
        vec![]
    }
}
impl<T: Tokenize, D: MatchDirection> Searcher<T, D> {
    pub fn new(graph: HypergraphRef<T>) -> Self {
        Self {
            graph,
            _ty: Default::default(),
        }
    }
    // find largest matching direct parent
    pub(crate) fn find_pattern_parent(
        &self,
        pattern: impl IntoPattern,
    ) -> SearchResult {
        self.bft_search::<ParentSearch<T, D>, _>(
            pattern,
        )
    }
    /// find largest matching ancestor for pattern
    pub(crate) fn find_pattern_ancestor(
        &self,
        pattern: impl IntoPattern,
    ) -> SearchResult {
        self.bft_search::<AncestorSearch<T, D>, _>(
            pattern,
        )
    }
    fn bft_search<
        S: SearchTraversalPolicy<T, D> + Send,
        P: IntoPattern,
    >(
        &self,
        query: P,
    ) -> SearchResult {
        self.search::<Bft<T, D, Self, QueryRangePath, BaseResult, S>, S, _>(
            query,
        )
    }
    #[allow(unused)]
    fn search<
        'a,
        Ti: TraversalIterator<'a, T, D, Self, QueryRangePath, S, BaseResult> + Send,
        S: SearchTraversalPolicy<T, D>,
        P: IntoPattern,
    >(
        &'a self,
        query: P,
    ) -> SearchResult {
        let query_path = QueryRangePath::new_directed::<D, _>(query.borrow())
            .map_err(|(err, _)| err)?;
        let mut acc = ControlFlow::Continue(None);
        let mut stream = Ti::new(self, TraversalNode::query_node(query_path));

        while let Some((_depth, node)) = stream.next() {
            match <S::Folder as TraversalFolder<_, _, _, BaseResult>>::fold_found(self, acc.continue_value().unwrap(), node) {
                ControlFlow::Continue(c) => {
                    acc = ControlFlow::Continue(c);
                },
                ControlFlow::Break(found) => {
                    acc = ControlFlow::Break(found);
                    break;
                },
            };
        }
        match acc {
            ControlFlow::Continue(None) => Err(NoMatch::NotFound),
            ControlFlow::Continue(Some(found)) |
            ControlFlow::Break(found) => Ok(found)
        }
    }
    //#[allow(unused)]
    //fn par_search<
    //    Ti: TraversalIterator<T, D, Self, QueryRangePath, S, BaseResult> + Send,
    //    S: SearchTraversalPolicy<T, D>,
    //    P: IntoPattern,
    //>(
    //    &'a self,
    //    query: P,
    //) -> SearchResult {
    //    let query_path = QueryRangePath::new_directed::<D, _>(query.borrow())
    //        .map_err(|(err, _)| err)?;
    //    match ParallelIterator::reduce(
    //        Ti::new(self, TraversalNode::query_node(query_path))
    //            .par_bridge()
    //            .try_fold_with(None, |acc, (_depth, node)|
    //                tokio::runtime::Handle::current().block_on(
    //                    <S::Folder as TraversalFolder<_, _, _, BaseResult>>::fold_found(self, acc, node)
    //                )
    //            ),
    //            || ControlFlow::Continue(None), |a, b|
    //                match (a, b) {
    //                    (ControlFlow::Break(b), ControlFlow::Continue(_)) |
    //                    (ControlFlow::Continue(_), ControlFlow::Break(b)) =>
    //                        ControlFlow::Break(b),
    //                    (ControlFlow::Break(a), ControlFlow::Break(b)) =>
    //                        ControlFlow::Break(
    //                            std::cmp::max_by(
    //                                a,
    //                                b,
    //                                |a, b|
    //                                    a.found.cmp(&b.found)
    //                            )
    //                        ),
    //                    (ControlFlow::Continue(a), ControlFlow::Continue(b)) =>
    //                        ControlFlow::Continue(match (a, b) {
    //                            (None, None) => None,
    //                            (None, Some(found)) |
    //                            (Some(found), None) => Some(found),
    //                            (Some(a), Some(b)) =>
    //                                Some(std::cmp::max_by(a, b, |a, b| a.found.cmp(&b.found)))
    //                        })
    //                }
    //        )
    //    {
    //        ControlFlow::Continue(None) => Err(NoMatch::NotFound),
    //        ControlFlow::Continue(Some(found)) |
    //        ControlFlow::Break(found) => Ok(found)
    //    }
    //}
}
