use std::sync::{
    RwLockReadGuard,
    RwLockWriteGuard,
};

use hypercontext_api::{
    graph::{
        vertex::child::Child,
        Hypergraph,
        HypergraphRef,
    },
    interval::cache::IntervalGraph,
    join::context::JoinContext,
    path::structs::rooted::pattern_range::PatternRangePath,
    traversal::{
        container::bft::BftQueue,
        fold::{
            state::FoldState,
            ErrorState,
            Foldable,
        },
        iterator::policy::DirectedTraversalPolicy,
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

#[derive(Debug)]
pub struct InsertPolicy {}

// <'a: 'g, 'g>
impl DirectedTraversalPolicy for InsertPolicy {
    type Trav = InsertContext;
}

pub trait InsertTraversalPolicy: DirectedTraversalPolicy<Trav = InsertContext> {}

impl InsertTraversalPolicy for InsertPolicy {}

#[derive(Debug)]
pub struct InsertTraversal;

impl TraversalKind for InsertTraversal {
    type Trav = InsertContext;
    type Container = BftQueue;
    type Policy = InsertPolicy;
}

#[derive(Debug, Clone)]
pub struct InsertContext {
    pub graph: HypergraphRef,
}
impl InsertContext {
    pub fn new(graph: HypergraphRef) -> Self {
        Self { graph }
    }
    pub fn insert(
        &mut self,
        foldable: impl Foldable,
    ) -> Result<(Child, PatternRangePath), ErrorState> {
        match foldable.fold::<InsertTraversal>(self.clone()) {
            Ok(result) => match result.result {
                FoundRange::Complete(c, p) => Ok((c, p)),
                FoundRange::Incomplete(fold_state) => Ok(self.insert_state(fold_state)),
            },
            Err(err) => Err(err),
        }
    }
    pub fn insert_state(
        &mut self,
        mut fold_state: FoldState,
    ) -> (Child, PatternRangePath) {
        let interval = IntervalGraph::new(&mut self.graph_mut(), &mut fold_state);
        let mut ctx = JoinContext {
            trav: self.graph.clone(),
            interval,
        };
        let new_index = ctx.join_subgraph();
        (new_index, fold_state.end_state.cursor.path)
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
