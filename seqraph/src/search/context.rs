use hypercontext_api::{
    graph::getters::ErrorReason,
    traversal::{
        container::bft::BftQueue,
        fold::Foldable,
        iterator::policy::{
            AncestorPolicy,
            ParentPolicy,
        },
        result::FinishedState,
        traversable::Traversable,
        TraversalKind,
    },
};

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
#[derive(Debug, Default)]
pub struct AncestorSearchTraversal<T: Traversable>(std::marker::PhantomData<T>);

impl<T: Traversable> TraversalKind for AncestorSearchTraversal<T> {
    type Trav = SearchContext<T>;
    type Container = BftQueue;
    type Policy = AncestorPolicy<Self::Trav>;
}
#[derive(Debug, Default)]
pub struct ParentSearchTraversal<T: Traversable>(std::marker::PhantomData<T>);

impl<T: Traversable> TraversalKind for ParentSearchTraversal<T> {
    type Trav = SearchContext<T>;
    type Container = BftQueue;
    type Policy = ParentPolicy<Self::Trav>;
}
impl<T: Traversable> SearchContext<T> {
    pub fn new(graph: T) -> Self {
        Self { graph }
    }
    // find largest matching direct parent
    pub fn find_pattern_parent(
        self,
        foldable: impl Foldable,
    ) -> SearchResult {
        self.search::<_, ParentSearchTraversal<T>>(foldable)
    }
    /// find largest matching ancestor for pattern
    pub fn find_pattern_ancestor(
        self,
        foldable: impl Foldable,
    ) -> SearchResult {
        self.search::<_, AncestorSearchTraversal<T>>(foldable)
    }
    //, Ti: TraversalIterator<'a, Trav = Self>
    fn search<F: Foldable, K: TraversalKind<Trav = Self>>(
        self,
        foldable: F,
    ) -> SearchResult {
        foldable.fold::<K>(self).map_err(|err| err.reason)
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
