use crate::{
    graph::HypergraphRef,
    search::NoMatch,
    traversal::{
        folder::{
            state::FoldResult,
            TraversalFolder,
        },
        iterator::traverser::bft::Bft,
        path::structs::query_range_path::QueryRangePath,
        policy::DirectedTraversalPolicy,
    },
    vertex::{
        child::Child,
        pattern::IntoPattern,
    },
};

#[derive(Debug, Clone)]
pub struct Indexer {
    pub graph: HypergraphRef,
}

#[derive(Debug)]
pub struct IndexingPolicy {}

impl<'a: 'g, 'g> DirectedTraversalPolicy for IndexingPolicy {
    type Trav = Indexer;

    //#[instrument(skip(trav, primer))]
    //fn at_postfix(
    //    trav: &Self::Trav,
    //    primer: Primer,
    //) -> Postfix {
    //    let trav = trav.clone();
    //    let path = primer.role_path();
    //    println!("after end match {:?}", path);
    //    // index postfix of match

    //    let match_end =
    //        if let Some(IndexSplitResult {
    //            inner: post,
    //            location: entry,
    //            ..
    //        }) = trav
    //        .pather::<IndexBack>()
    //        .index_primary_path::<InnerSide, _>(
    //            path.path().into_iter().chain(
    //                std::iter::once(&path.child_location())
    //            ).collect_vec(),
    //        ) {
    //            MatchEnd::Path(RolePath {
    //                path: vec![entry],
    //                child: post,
    //                width: post.width(),
    //                token_pos: trav.graph().expect_pattern_range_width(entry, 0..entry.sub_index),
    //                _ty: Default::default(),
    //            })
    //        } else {
    //            MatchEnd::Complete(path.child_location().parent)
    //        };
    //    println!("after end match result {:?}", match_end);
    //    BaseResult::into_postfix(primer, match_end)
    //}
}

pub trait IndexerTraversalPolicy: DirectedTraversalPolicy<Trav=Indexer> {}

impl<'a: 'g, 'g> IndexerTraversalPolicy for IndexingPolicy {}

impl TraversalFolder for Indexer {
    //type Break = TraversalResult<R, Q>;
    //type Continue = TraversalResult<R, Q>;
    //type Result = (<R as ResultKind>::Indexed, Q);
    type Iterator<'a> = Bft<'a, Self, IndexingPolicy>;

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

impl Indexer {
    pub fn new(graph: HypergraphRef) -> Self {
        Self { graph }
    }
    //pub fn contexter<Side: IndexSide<<BaseGraphKind as GraphKind>::Direction>>(&self) -> Contexter<Side> {
    //    Contexter::new(self.clone())
    //}
    //pub fn splitter<Side: IndexSide<<BaseGraphKind as GraphKind>::Direction>>(&self) -> Splitter<Side> {
    //    Splitter::new(self.clone())
    //}
    //pub fn pather<Side: IndexSide<<BaseGraphKind as GraphKind>::Direction>>(&self) -> Pather<Side> {
    //    Pather::new(self.clone())
    //}
    pub fn index_pattern(
        &mut self,
        query: impl IntoPattern,
    ) -> Result<(Child, QueryRangePath), NoMatch> {
        match <Self as TraversalFolder>::fold_query(self, query) {
            Ok(result) => match result.result {
                FoldResult::Complete(c) => Ok((c, result.query)),
                FoldResult::Incomplete(s) => Ok((self.join_subgraph(s), result.query)),
            },
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
