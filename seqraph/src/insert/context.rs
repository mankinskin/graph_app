use std::{
    ops::ControlFlow,
    sync::{
        RwLockReadGuard,
        RwLockWriteGuard,
    },
};

use hypercontext_api::{
    graph::{
        getters::ErrorReason, vertex::{
            child::Child,
            pattern::IntoPattern,
        }, Hypergraph, HypergraphRef
    },
    join::context::JoinContext,
    path::structs::query_range_path::{
        QueryPath,
        QueryRangePath,
    },
    split::cache::split::Split,
    traversal::{
        cache::key::SplitKey,
        container::bft::Bft,
        fold::{state::FoldState, ErrorState, FoldContext},
        iterator::policy::DirectedTraversalPolicy,
        result::{kind::BaseResult, FoundRange},
        traversable::{
            impl_traversable,
            impl_traversable_mut,
            Traversable,
            TraversableMut,
        },
    },
    HashMap,
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

//impl TraversalFolder for InsertContext {
//    type Iterator<'a> = Bft<'a, Self, InsertPolicy>;
//}

impl InsertContext {
    pub fn new(graph: HypergraphRef) -> Self {
        Self { graph }
    }
    pub fn join(
        &self,
        fold_state: &mut FoldState,
    ) -> JoinContext {
        JoinContext::new(self.clone().graph_mut(), fold_state)
    }
    pub fn insert_pattern(
        &mut self,
        query: impl IntoPattern,
    ) -> Result<(Child, QueryRangePath), ErrorReason> {
        match FoldContext::fold_pattern(self, query) {
            Ok(result) => match result.result {
                FoundRange::Complete(c) => Ok((c, result.query.path)),
                FoundRange::Incomplete(fold_state) => Ok((self.join(&mut fold_state).join_subgraph(), result.query.path)),
            },
            Err(ErrorState {
                reason: ErrorReason::SingleIndex(c),
                query,
                found: Some(FoundRange::Complete(found)),
            }) => Ok((c, found)),
            Err((err, _)) => Err(err),
        }
    }
    //pub fn index_query<Q: QueryPath>(
    //    &mut self,
    //    query: Q,
    //) -> Result<(Child, Q), ErrorReason> {
    //    self.index_result_kind::<BaseResult, _>(query)
    //}
    //pub fn index_query_with_origin<
    //    Q: QueryPath,
    //>(
    //    &mut self,
    //    query: Q,
    //) -> Result<(OriginPath<Child>, Q), ErrorReason> {
    //    self.index_result_kind::<OriginPathResult, _>(query)
    //}
    //pub fn index_result_kind<R: ResultKind, Q: QueryPath>(
    //    &mut self,
    //    query: Q,
    //) -> Result<(<R as ResultKind>::Indexed, Q), ErrorReason> {
    //    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    //    query.hash(&mut hasher);
    //    let _h = hasher.finish();
    //    let acc =
    //        self.run_indexing::<R, _, InsertPolicy<T, D, Q, _>, Bft<_, _, _, _, _, _>>(query)?;
    //    self.finish_result::<R, Q>(acc)
    //}
    //fn run_indexing<
    //    'a,
    //    R: ResultKind,
    //    Q: QueryPath,
    //    S: InsertTraversalPolicy<T, D, Q, R>,
    //    Ti: TraversalIterator<'a, T, D, Self, Q, InsertTraversalPolicy<T, D, Q, R>, R>,
    //>(
    //    &'a mut self,
    //    query_path: Q,
    //) -> Result<ControlFlow<(<R as ResultKind>::Indexed, Q), Option<FinishedState>>, ErrorReason>
    //{
    //    let mut acc = ControlFlow::Continue(None);
    //    let mut stream = Ti::new(self, query_path).ok_or(ErrorReason::EmptyPatterns)?;
    //    while let Some((_depth, node)) = stream.next() {
    //        match FoldContext::fold_found(self, acc.continue_value().unwrap(), node) {
    //            ControlFlow::Continue(c) => {
    //                acc = ControlFlow::Continue(c);
    //            }
    //            ControlFlow::Break(found) => {
    //                acc = ControlFlow::Break(found);
    //                break;
    //            }
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
    impl for &'_ InsertContext,
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
