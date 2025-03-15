use std::sync::{
    RwLockReadGuard,
    RwLockWriteGuard,
};

use derive_more::From;
use hypercontext_api::{
    graph::{
        getters::ErrorReason,
        vertex::child::Child,
        Hypergraph,
        HypergraphRef,
    },
    interval::{
        InitInterval,
        IntervalGraph,
    },
    join::context::JoinContext,
    traversal::{
        container::bft::BftQueue,
        fold::foldable::{
            ErrorState,
            Foldable,
        },
        result::FoundRange,
        traversable::{
            impl_traversable,
            impl_traversable_mut,
            Traversable,
            TraversableMut,
        },
        TraversalKind,
    },
};

use crate::search::context::ParentPolicy;

#[derive(Debug, Clone, Default)]
pub struct InsertTraversal;

impl TraversalKind for InsertTraversal {
    type Trav = InsertContext;
    type Container = BftQueue;
    type Policy = ParentPolicy<Self::Trav>;
}
#[derive(Debug, Clone, From)]
pub struct InsertContext {
    graph: HypergraphRef,
}

impl InsertContext {
    pub fn insert(
        &mut self,
        foldable: impl Foldable,
    ) -> Result<Child, ErrorState> {
        match foldable.fold::<InsertTraversal>(self.clone()) {
            Ok(result) => match result.result {
                FoundRange::Complete(c) => Ok(c),
                FoundRange::Incomplete(fold_state) => {
                    Ok(self.insert_init(InitInterval::from(fold_state)))
                }
            },
            Err(err) => Err(err),
        }
    }
    pub fn insert_or_get_complete(
        &mut self,
        query: impl Foldable,
    ) -> Result<Child, ErrorReason> {
        match self.insert(query) {
            Err(ErrorState {
                reason: ErrorReason::SingleIndex(c),
                found: Some(FoundRange::Complete(_)),
            }) => Ok(c),
            Err(err) => Err(err.reason),
            Ok(v) => Ok(v),
        }
    }
    pub fn insert_init(
        &mut self,
        init: InitInterval,
    ) -> Child {
        let interval = IntervalGraph::from((&mut self.graph_mut(), init));
        let mut ctx = JoinContext {
            trav: self.graph.clone(),
            interval,
        };
        ctx.join_subgraph()
    }
}

impl_traversable! {
    impl for InsertContext,
    self => self.graph.read().unwrap();
    <'a> RwLockReadGuard<'a, Hypergraph>
}
impl_traversable_mut! {
    impl for InsertContext,
    self => self.graph.write().unwrap();
    <'a> RwLockWriteGuard<'a, Hypergraph>
}
