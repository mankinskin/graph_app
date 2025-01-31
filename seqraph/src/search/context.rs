use hypercontext_api::{
    graph::{
        getters::ErrorReason,
        vertex::pattern::IntoPattern,
    },
    traversal::{
        container::bft::BftQueue,
        fold::FoldContext,
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
        &self,
        pattern: impl IntoPattern,
    ) -> SearchResult {
        self.search::<_, ParentSearchTraversal<T>>(pattern)
    }
    /// find largest matching ancestor for pattern
    pub fn find_pattern_ancestor(
        &self,
        pattern: impl IntoPattern,
    ) -> SearchResult {
        self.search::<_, AncestorSearchTraversal<T>>(pattern)
    }
    //, Ti: TraversalIterator<'a, Trav = Self>
    fn search<P: IntoPattern, K: TraversalKind<Trav = Self>>(
        &self,
        query: P,
    ) -> SearchResult {
        FoldContext::<K>::fold_pattern(self, query).map_err(|err| err.reason)
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
