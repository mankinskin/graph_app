use crate::{
    graph::HypergraphRef, join::{context::JoinContext, splits::SplitFrontier}, search::NoMatch, split::cache::{split::Split, SplitCache}, traversal::{
        cache::key::SplitKey, folder::{
            state::{FoldResult, FoldState},
            TraversalFolder,
        }, iterator::traverser::bft::Bft, path::structs::query_range_path::QueryRangePath, policy::DirectedTraversalPolicy, traversable::TraversableMut
    }, HashMap
};
use crate::graph::vertex::{
    child::Child,
    pattern::IntoPattern,
};

#[derive(Debug, Clone)]
pub struct InsertContext {
    pub graph: HypergraphRef,
}

#[derive(Debug)]
pub struct InsertPolicy {}

// <'a: 'g, 'g>
impl DirectedTraversalPolicy for InsertPolicy {
    type Trav = InsertContext;
}

pub trait InsertTraversalPolicy: DirectedTraversalPolicy<Trav = InsertContext> {}

impl InsertTraversalPolicy for InsertPolicy {}

impl TraversalFolder for InsertContext {
    type Iterator<'a> = Bft<'a, Self, InsertPolicy>;
}



impl InsertContext {
    pub fn new(graph: HypergraphRef) -> Self {
        Self { graph }
    }
    pub fn join(&self, final_splits: &HashMap<SplitKey, Split>) -> JoinContext {
        JoinContext::new(self.clone().graph_mut(), final_splits)
    }
    pub fn insert_pattern(
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
    pub fn join_subgraph(
        &mut self,
        fold_state: FoldState,
    ) -> Child {
        let root = fold_state.root;
        let split_cache = SplitCache::new(self, fold_state);

        let final_splits = SplitFrontier::new(split_cache.leaves.iter().cloned().rev())
            .join_final_splits(root, &split_cache);

        let root_mode = split_cache.root_mode;
        let x = self.join(&final_splits)
            .node(root, &split_cache)
            .join_root_partitions(root_mode);
        x
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

impl_traversable! {
    impl for InsertContext,
    self => self.graph.read().unwrap();
    <'a> RwLockReadGuard<'a, Hypergraph>
}
impl_traversable! {
    impl for &'_ mut InsertContext,
    self => self.graph.read().unwrap();
    <'a> RwLockReadGuard<'a, Hypergraph>
}
impl_traversable_mut! {
    impl for InsertContext,
    self => self.graph.write().unwrap();
    <'a> RwLockWriteGuard<'a, Hypergraph>
}
impl_traversable_mut! {
    impl for &'_ mut InsertContext,
    self => self.graph.write().unwrap();
    <'a> RwLockWriteGuard<'a, Hypergraph>
}
