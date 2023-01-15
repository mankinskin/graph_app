use std::hash::Hasher;

use crate::*;
use super::*;

#[derive(Debug, Clone)]
pub struct Indexer<T: Tokenize, D: IndexDirection> {
    pub graph: HypergraphRef<T>,
    _ty: std::marker::PhantomData<D>,
}
pub struct IndexingPolicy<T: Tokenize, D: IndexDirection, Q: QueryPath, R: ResultKind> {
    _ty: std::marker::PhantomData<(T, D, Q, R)>,
}
impl<
    'a: 'g,
    'g,
    T: Tokenize,
    D: IndexDirection,
    Q: QueryPath,
    R: ResultKind,
>
DirectedTraversalPolicy<T, D, Q, R> for IndexingPolicy<T, D, Q, R>
{
    type Trav = Indexer<T, D>;

    #[instrument(skip(trav, primer))]
    fn at_postfix(
        trav: &Self::Trav,
        primer: R::Primer,
    ) -> R::Postfix {
        let trav = trav.clone();
        let path = primer.role_path();
        println!("after end match {:?}", path);
        // index postfix of match

        let match_end =
            if let Some(IndexSplitResult {
                inner: post,
                location: entry,
                ..
            }) = trav
            .pather::<IndexBack>()
            .index_primary_path::<InnerSide, _>(
                path.path().into_iter().chain(
                    std::iter::once(&path.child_location())
                ).collect_vec(),
            ) {
                MatchEnd::Path(ChildPath {
                    path: vec![entry],
                    child: post,
                    width: post.width(),
                    token_pos: trav.graph().expect_pattern_range_width(entry, 0..entry.sub_index),
                    _ty: Default::default(),
                })
            } else {
                MatchEnd::Complete(path.child_location().parent)
            };
        println!("after end match result {:?}", match_end);
        R::into_postfix(primer, match_end)
    }
}

pub trait IndexerTraversalPolicy<
    T: Tokenize,
    D: IndexDirection,
    Q: QueryPath,
    R: ResultKind,
>:
    DirectedTraversalPolicy<
        T, D, Q, R,
        Trav=Indexer<T, D>,
    >
{
}
impl<
    'a: 'g,
    'g,
    T: Tokenize,
    D: IndexDirection,
    Q: QueryPath,
    R: ResultKind,
> IndexerTraversalPolicy<T, D, Q, R> for IndexingPolicy<T, D, Q, R>
{}

impl<T, D, Q, R> TraversalFolder<T, D, Q, IndexingPolicy<T, D, Q, R>, R> for Indexer<T, D>
where 
    T: Tokenize,
    D: IndexDirection,
    Q: QueryPath,
    R: ResultKind,
{
    //type Break = TraversalResult<R, Q>;
    //type Continue = TraversalResult<R, Q>;
    //type Result = (<R as ResultKind>::Indexed, Q);
    type NodeCollection = BftQueue<R, Q>;

    //fn map_state(
    //    &self,
    //    acc: ControlFlow<Self::Break, Self::Continue>,
    //    node: TraversalState<R, Q>,
    //) -> ControlFlow<Self::Break, Self::Continue> {
    //    match node {
    //        TraversalState::QueryEnd(_, _, res) => {
    //            //println!("fold query end {:#?}", res);
    //            ControlFlow::Break()
    //        },
    //        TraversalState::Mismatch(_, _, res) => {
    //            //println!("fold mismatch {:#?}", res);
    //            ControlFlow::Continue(search::pick_max_result::<R, _>(acc, res))
    //        },
    //        TraversalState::MatchEnd(_, _, postfix, query) => {
    //            //println!("fold match end {:#?}", postfix);
    //            //let found = match_end
    //            //    .into_range_path().into_result(query);
    //            //if let Some(r) = found.found.get_range() {
    //            //    assert!(r.root_child_pos() != r.root_child_pos());
    //            //}
    //            ControlFlow::Continue(search::pick_max_result::<R, _>(
    //                acc,
    //                <R as ResultKind>::Found::from(postfix).into_result(query)
    //            ))
    //        },
    //        _ => ControlFlow::Continue(acc)
    //    }
    //}
    //fn map_continue(
    //    &self,
    //    cont: Self::Continue,
    //) -> Self::Result {
    //    (
    //        R::index_found::<_, D>(cont.path, self),
    //        cont.query
    //    )
    //}

    //fn map_break(
    //    &self,
    //    brk: Self::Break,
    //) -> Self::Result {
    //    let mut trav = self.clone();
    //    (
    //        R::index_found::<_, D>(
    //            brk.path,
    //            &mut trav,
    //        ),
    //        brk.query,
    //    )
    //}
}

impl<T: Tokenize, D: IndexDirection> Indexer<T, D> {
    pub fn new(graph: HypergraphRef<T>) -> Self {
        Self {
            graph,
            _ty: Default::default(),
        }
    }
    pub fn contexter<Side: IndexSide<D>>(&self) -> Contexter<T, D, Side> {
        Contexter::new(self.clone())
    }
    pub fn splitter<Side: IndexSide<D>>(&self) -> Splitter<T, D, Side> {
        Splitter::new(self.clone())
    }
    pub fn pather<Side: IndexSide<D>>(&self) -> Pather<T, D, Side> {
        Pather::new(self.clone())
    }
    pub fn index_pattern(
        &mut self,
        query: impl IntoPattern,
    ) -> Result<(Child, QueryRangePath), NoMatch> {
        match <S::Folder as TraversalFolder<_, D, _, R>>::fold_query(query) {
            Ok((result, remaining_query)) => Ok((self.index_found(result), remaining_query)),
            Err((NoMatch::SingleIndex(c), path)) => Ok((c, path)),
            Err((err, _)) => Err(err),
        }
    }
    //pub fn index_query<
    //    Q: QueryPath,
    //>(
    //    &mut self,
    //    query: Q,
    //) -> Result<(Child, Q), NoMatch> {
    //    self.index_result_kind::<BaseResult, _>(query)
    //}
    //pub fn index_query_with_origin<
    //    Q: QueryPath,
    //>(
    //    &mut self,
    //    query: Q,
    //) -> Result<(OriginPath<Child>, Q), NoMatch> {
    //    self.index_result_kind::<OriginPathResult, _>(query)
    //}
    //pub fn index_result_kind<
    //    R: ResultKind,
    //    Q: QueryPath,
    //>(
    //    &mut self,
    //    query: Q,
    //) -> Result<(<R as ResultKind>::Indexed, Q), NoMatch> {
    //    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    //    query.hash(&mut hasher);
    //    let _h = hasher.finish();
    //    let acc = self.run_indexing::<R, _, IndexingPolicy<T, D, Q, _>, Bft<_, _, _, _, _, _>>(query)?;
    //    self.finish_result::<R, Q>(acc)
    //}
    //fn run_indexing<
    //    'a,
    //    R: ResultKind,
    //    Q: QueryPath,
    //    S: IndexerTraversalPolicy<T, D, Q, R>,
    //    Ti: TraversalIterator<'a, T, D, Self, Q, IndexingTraversalPolicy<T, D, Q, R>, R>,
    //>(
    //    &'a mut self,
    //    query_path: Q,
    //) -> Result<ControlFlow<(<R as ResultKind>::Indexed, Q), Option<TraversalResult<R, Q>>>, NoMatch> {
    //    let mut acc = ControlFlow::Continue(None);
    //    let mut stream = Ti::new(self, query_path)
    //        .ok_or(NoMatch::EmptyPatterns)?;
    //    while let Some((_depth, node)) = stream.next() {
    //        match <S::Folder as TraversalFolder<_, _, _, R>>::fold_found(self, acc.continue_value().unwrap(), node) {
    //            ControlFlow::Continue(c) => {
    //                acc = ControlFlow::Continue(c);
    //            },
    //            ControlFlow::Break(found) => {
    //                acc = ControlFlow::Break(found);
    //                break;
    //            },
    //        };
    //    }
    //    Ok(acc)
    //}
}