use std::convert::TryFrom;

use crate::{
    interval::{
        IntervalGraph,
        init::InitInterval,
    },
    join::context::JoinContext,
};
use context_search::{
    search::context::AncestorPolicy,
    traversal::{
        TraversalKind,
        container::bft::BftQueue,
        fold::foldable::{
            ErrorState,
            Foldable,
        },
        result::{
            CompleteState,
            FinishedKind,
        },
    },
};
use context_trace::{
    graph::{
        Hypergraph,
        HypergraphRef,
        getters::ErrorReason,
        vertex::child::Child,
    },
    impl_has_graph,
    impl_has_graph_mut,
    trace::has_graph::HasGraphMut,
};
use derive_more::From;
use std::sync::{
    RwLockReadGuard,
    RwLockWriteGuard,
};

#[derive(Debug, Clone, Default)]
pub struct InsertTraversal;

impl TraversalKind for InsertTraversal {
    type Trav = InsertContext;
    type Container = BftQueue;
    type Policy = AncestorPolicy<Self::Trav>;
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
            Ok(result) => match CompleteState::try_from(result) {
                Ok(state) => Ok(state.root),
                Err(state) => Ok(self.insert_init(InitInterval::from(state))),
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
                found: Some(FinishedKind::Complete(_)),
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
        JoinContext {
            trav: self.graph.clone(),
            interval,
        }
        .join_subgraph()
    }
}
impl_has_graph! {
    impl for InsertContext,
    self => self.graph.read().unwrap();
    <'a> RwLockReadGuard<'a, Hypergraph>
}
impl_has_graph_mut! {
    impl for InsertContext,
    self => self.graph.write().unwrap();
    <'a> RwLockWriteGuard<'a, Hypergraph>
}
