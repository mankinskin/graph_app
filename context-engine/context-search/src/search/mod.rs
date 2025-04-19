use crate::traversal::fold::foldable::Foldable;
use context::{
    AncestorSearchTraversal,
    ParentSearchTraversal,
    SearchContext,
    SearchResult,
};
use context_trace::{
    graph::{
        kind::TokenOf,
        vertex::token::{
            tokenizing_iter,
            AsToken,
        },
        Hypergraph,
        HypergraphRef,
    },
    trace::has_graph::{
        TravKind,
        HasGraph,
    },
};
pub mod context;

#[allow(dead_code)]
pub trait Searchable: HasGraph {
    fn ctx(&self) -> SearchContext<Self>;
    fn find_sequence(
        &self,
        pattern: impl IntoIterator<Item = impl AsToken<TokenOf<TravKind<Self>>>>,
    ) -> SearchResult {
        let iter = tokenizing_iter(pattern.into_iter());
        let pattern = self.graph().get_token_children(iter)?;
        self.find_ancestor(pattern)
    }
    // find largest matching direct parent
    fn find_parent(
        &self,
        foldable: impl Foldable,
    ) -> SearchResult {
        foldable
            .fold::<ParentSearchTraversal<Self>>(self.ctx())
            .map_err(|err| err.reason)
    }
    /// find largest matching ancestor for pattern
    fn find_ancestor(
        &self,
        foldable: impl Foldable,
    ) -> SearchResult {
        foldable
            .fold::<AncestorSearchTraversal<Self>>(self.ctx())
            .map_err(|err| err.reason)
    }
}

impl Searchable for &Hypergraph {
    fn ctx(&self) -> SearchContext<Self> {
        SearchContext::new(self)
    }
}

impl Searchable for HypergraphRef {
    fn ctx(&self) -> SearchContext<Self> {
        SearchContext::new(self.clone())
    }
}
