use crate::{
    fold::result::FinishedState,
    traversal::{
        container::bft::BftQueue,
        policy::DirectedTraversalPolicy,
        TraversalKind,
    },
};
use context_trace::{
    graph::getters::ErrorReason,
    trace::{
        has_graph::HasGraph,
        state::parent::{
            ParentBatch,
            ParentState,
        },
    },
};
#[derive(Debug)]
pub struct AncestorPolicy<T: HasGraph>(std::marker::PhantomData<T>);

impl<T: HasGraph> DirectedTraversalPolicy for AncestorPolicy<T> {
    type Trav = T;
}

#[derive(Debug)]
pub struct ParentPolicy<T: HasGraph>(std::marker::PhantomData<T>);

impl<T: HasGraph> DirectedTraversalPolicy for ParentPolicy<T> {
    type Trav = T;
    fn next_batch(
        _trav: &Self::Trav,
        _state: &ParentState,
    ) -> Option<ParentBatch> {
        None
    }
}

#[derive(Clone, Debug)]
pub struct SearchCtx<T: HasGraph> {
    pub graph: T,
}

pub type SearchResult = Result<FinishedState, ErrorReason>;
#[derive(Debug)]
pub struct AncestorSearchTraversal<T: HasGraph>(std::marker::PhantomData<T>);
impl<T: HasGraph> Default for AncestorSearchTraversal<T> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<T: HasGraph> TraversalKind for AncestorSearchTraversal<T> {
    type Trav = SearchCtx<T>;
    type Container = BftQueue;
    type Policy = AncestorPolicy<Self::Trav>;
}
#[derive(Debug)]
pub struct ParentSearchTraversal<T: HasGraph>(std::marker::PhantomData<T>);
impl<T: HasGraph> Default for ParentSearchTraversal<T> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<T: HasGraph> TraversalKind for ParentSearchTraversal<T> {
    type Trav = SearchCtx<T>;
    type Container = BftQueue;
    type Policy = ParentPolicy<Self::Trav>;
}
impl<T: HasGraph> SearchCtx<T> {
    pub fn new(graph: T) -> Self {
        Self { graph }
    }
}

impl<T: HasGraph> HasGraph for SearchCtx<T> {
    type Kind = T::Kind;
    type Guard<'a>
        = T::Guard<'a>
    where
        T: 'a;
    fn graph(&self) -> Self::Guard<'_> {
        self.graph.graph()
    }
}

//impl<'g, T: HasGraph> HasGraph for &'g SearchCtx<T> {
//    type Kind = T::Kind;
//    type Guard<'a> = T::Guard<'a> where T: 'a, 'g: 'a;
//    fn graph(&self) -> Self::Guard<'_> {
//        self.graph.graph()
//    }
//}
