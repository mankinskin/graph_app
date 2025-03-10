use hypercontext_api::{
    graph::getters::ErrorReason,
    traversal::{
        container::bft::BftQueue,
        iterator::policy::DirectedTraversalPolicy,
        result::FinishedState,
        state::bottom_up::parent::ParentState,
        traversable::Traversable,
        TraversalKind,
    },
};

#[derive(Debug)]
pub struct AncestorPolicy<T: Traversable>(std::marker::PhantomData<T>);

impl<T: Traversable> DirectedTraversalPolicy for AncestorPolicy<T> {
    type Trav = T;
}

#[derive(Debug)]
pub struct ParentPolicy<T: Traversable>(std::marker::PhantomData<T>);

impl<T: Traversable> DirectedTraversalPolicy for ParentPolicy<T> {
    type Trav = T;
    fn next_parents(
        _trav: &Self::Trav,
        _state: &ParentState,
    ) -> Vec<ParentState> {
        vec![]
    }
}

#[derive(Clone, Debug)]
pub struct SearchContext<T: Traversable> {
    pub graph: T,
}

//pub trait SearchTraversalPolicy<T: Traversable>:
//    DirectedTraversalPolicy<Trav = T>
//{
//}
//
//impl<T: Traversable> SearchTraversalPolicy<T> for AncestorPolicy<T> {}
//
//impl<T: Traversable> SearchTraversalPolicy<T> for ParentPolicy<T> {}

pub type SearchResult = Result<FinishedState, ErrorReason>;
#[derive(Debug)]
pub struct AncestorSearchTraversal<T: Traversable>(std::marker::PhantomData<T>);
impl<T: Traversable> Default for AncestorSearchTraversal<T> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<T: Traversable> TraversalKind for AncestorSearchTraversal<T> {
    type Trav = SearchContext<T>;
    type Container = BftQueue;
    type Policy = AncestorPolicy<Self::Trav>;
}
#[derive(Debug)]
pub struct ParentSearchTraversal<T: Traversable>(std::marker::PhantomData<T>);
impl<T: Traversable> Default for ParentSearchTraversal<T> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<T: Traversable> TraversalKind for ParentSearchTraversal<T> {
    type Trav = SearchContext<T>;
    type Container = BftQueue;
    type Policy = ParentPolicy<Self::Trav>;
}
impl<T: Traversable> SearchContext<T> {
    pub fn new(graph: T) -> Self {
        Self { graph }
    }
}

impl<T: Traversable> Traversable for SearchContext<T> {
    type Kind = T::Kind;
    type Guard<'a>
        = T::Guard<'a>
    where
        T: 'a;
    fn graph(&self) -> Self::Guard<'_> {
        self.graph.graph()
    }
}

//impl<'g, T: Traversable> Traversable for &'g SearchContext<T> {
//    type Kind = T::Kind;
//    type Guard<'a> = T::Guard<'a> where T: 'a, 'g: 'a;
//    fn graph(&self) -> Self::Guard<'_> {
//        self.graph.graph()
//    }
//}
