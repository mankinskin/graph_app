use crate::traversal::{
    container::bft::BftQueue,
    iterator::policy::DirectedTraversalPolicy,
    result::FinishedState,
    state::parent::{
        batch::ParentBatch,
        ParentState,
    },
    TraversalKind,
};
use context_trace::{
    graph::getters::ErrorReason,
    trace::has_graph::HasGraph,
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
pub struct SearchContext<T: HasGraph> {
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
    type Trav = SearchContext<T>;
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
    type Trav = SearchContext<T>;
    type Container = BftQueue;
    type Policy = ParentPolicy<Self::Trav>;
}
impl<T: HasGraph> SearchContext<T> {
    pub fn new(graph: T) -> Self {
        Self { graph }
    }
}

impl<T: HasGraph> HasGraph for SearchContext<T> {
    type Kind = T::Kind;
    type Guard<'a>
        = T::Guard<'a>
    where
        T: 'a;
    fn graph(&self) -> Self::Guard<'_> {
        self.graph.graph()
    }
}

//impl<'g, T: HasGraph> HasGraph for &'g SearchContext<T> {
//    type Kind = T::Kind;
//    type Guard<'a> = T::Guard<'a> where T: 'a, 'g: 'a;
//    fn graph(&self) -> Self::Guard<'_> {
//        self.graph.graph()
//    }
//}
