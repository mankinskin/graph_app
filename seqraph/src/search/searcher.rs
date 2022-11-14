use crate::{
    search::*,
    Hypergraph,
};
use std::ops::ControlFlow;
use async_std::sync::RwLockReadGuard;
//use rayon::iter::{
//    ParallelBridge,
//    ParallelIterator,
//};

#[derive(Clone, Debug)]
pub struct Searcher<T: Tokenize, D: MatchDirection> {
    graph: HypergraphRef<T>,
    _ty: std::marker::PhantomData<D>,
}
#[async_trait]
impl<'a: 'g, 'g, T: Tokenize + 'a, D: MatchDirection + 'a> Traversable<'a, 'g, T> for Searcher<T, D> {
    type Guard = RwLockReadGuard<'g, Hypergraph<T>>;
    async fn graph(&'g self) -> Self::Guard {
        self.graph.read().await
    }
}
#[async_trait]
impl<'a: 'g, 'g, T: Tokenize + 'a, D: MatchDirection + 'a> Traversable<'a, 'g, T> for &'a Searcher<T, D> {
    type Guard = RwLockReadGuard<'g, Hypergraph<T>>;
    async fn graph(&'g self) -> Self::Guard {
        self.graph.read().await
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
        BaseResult,
        Trav=Searcher<T, D>,
        Folder=Searcher<T, D>,
        //Primer=StartPath,
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

#[async_trait]
impl<'a: 'g, 'g, T: Tokenize + 'a, D: MatchDirection + 'a, Q: TraversalQuery + 'a, R: ResultKind + 'a>
    TraversalFolder<'a, 'g, T, D, Q, R> for Searcher<T, D>
{
    type Trav = Self;
    type Break = TraversalResult<<R as ResultKind>::Found, Q>;
    type Continue = Option<TraversalResult<<R as ResultKind>::Found, Q>>;

    async fn fold_found(
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
//    T: Tokenize + 'a,
//    D: MatchDirection + 'a,
//    Q: TraversalQuery,
//    R: ResultKind,
//    Folder: TraversalFolder<'a, 'g, T, D, Q, R, Continue=Option<TraversalResult<<R as ResultKind>::Found, Q>>>
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
#[async_trait]
impl<
    'a: 'g,
    'g,
    T: Tokenize + 'a,
    D: MatchDirection + 'a,
    R: ResultKind + 'a,
>
    DirectedTraversalPolicy<'a, 'g, T, D, QueryRangePath, R> for AncestorSearch<T, D>
{
    type Trav = Searcher<T, D>;
    type Folder = Searcher<T, D>;
    //type Primer = R::Result<StartPath>;

    async fn after_end_match(
        _trav: &'a Self::Trav,
        path: R::Primer,
    ) -> R::Postfix {
        R::Postfix::from(path)
    }
}
struct ParentSearch<T: Tokenize, D: MatchDirection> {
    _ty: std::marker::PhantomData<(T, D)>,
}
#[async_trait]
impl<
    'a: 'g,
    'g,
    T: Tokenize + 'a,
    D: MatchDirection + 'a,
    Q: TraversalQuery + 'a,
    R: ResultKind + 'a,
>
    DirectedTraversalPolicy<'a, 'g, T, D, Q, R> for ParentSearch<T, D>
{
    type Trav = Searcher<T, D>;
    type Folder = Searcher<T, D>;

    async fn after_end_match(
        _trav: &'a Self::Trav,
        path: R::Primer,
    ) -> R::Postfix {
        R::Postfix::from(path)
    }
    async fn next_parents(
        _trav: &'a Self::Trav,
        _query: &Q,
        _start: &R::Postfix,
    ) -> Vec<TraversalNode<R, Q>> {
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
    pub(crate) async fn find_pattern_parent(
        &'a self,
        pattern: impl IntoPattern,
    ) -> SearchResult {
        self.bft_search::<ParentSearch<T, D>, _>(
            pattern,
        ).await
    }
    /// find largest matching ancestor for pattern
    pub(crate) async fn find_pattern_ancestor(
        &'a self,
        pattern: impl IntoPattern,
    ) -> SearchResult {
        self.bft_search::<AncestorSearch<T, D>, _>(
            pattern,
        ).await
    }
    async fn bft_search<
        S: SearchTraversalPolicy<'a, 'g, T, D> + Send,
        P: IntoPattern,
    >(
        &'a self,
        query: P,
    ) -> SearchResult {
        self.search::<Bft<'a, 'g, T, D, Self, QueryRangePath, BaseResult, S>, S, _>(
            query,
        ).await
    }
    #[allow(unused)]
    async fn search<
        Ti: TraversalIterator<'a, 'g, T, D, Self, QueryRangePath, S, BaseResult> + Send,
        S: SearchTraversalPolicy<'a, 'g, T, D>,
        P: IntoPattern,
    >(
        &'a self,
        query: P,
    ) -> SearchResult {
        let query_path = QueryRangePath::new_directed::<D, _>(query.borrow())
            .map_err(|(err, _)| err)?;
        let mut acc = ControlFlow::Continue(None);
        let mut stream = pin!(Ti::new(self, TraversalNode::query_node(query_path)));

        while let Some((_depth, node)) = stream.next().await {
            match <S::Folder as TraversalFolder<_, _, _, BaseResult>>::fold_found(self, acc.continue_value().unwrap(), node).await {
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
    //    Ti: TraversalIterator<'a, 'g, T, D, Self, QueryRangePath, S, BaseResult> + Send,
    //    S: SearchTraversalPolicy<'a, 'g, T, D>,
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
