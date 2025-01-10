use hypercontext_api::{
    graph::{
        kind::TokenOf,
        vertex::{
            child::Child,
            pattern::{
                id::PatternId,
                Pattern,
            },
            token::{
                tokenizing_iter,
                AsToken,
            },
            VertexIndex,
        },
        Hypergraph,
        HypergraphRef,
    },
    traversal::traversable::{
        GraphKindOf,
        Traversable,
    },
};
use context::{SearchResult, SearchContext};

pub mod context;

pub trait Searchable: Traversable
{
    fn ctx(&self) -> SearchContext<Self>;
    //pub fn expect_pattern(
    //    &self,
    //    pattern: impl IntoIterator<Item = impl AsToken<T>>,
    //) -> Child {
    //    self.find_sequence(pattern).unwrap().unwrap_complete()
    //}
    fn find_ancestor(
        &self,
        pattern: impl IntoIterator<
            Item = impl hypercontext_api::graph::vertex::has_vertex_index::HasVertexIndex,
        >,
    ) -> SearchResult
    {
        let pattern = self.graph().to_children(pattern);
        self.ctx().find_pattern_ancestor(pattern)
    }
    fn find_parent(
        &self,
        pattern: impl IntoIterator<
            Item = impl hypercontext_api::graph::vertex::has_vertex_index::HasVertexIndex,
        >,
    ) -> SearchResult
    {
        let pattern = self.graph().to_children(pattern);
        self.ctx().find_pattern_parent(pattern)
    }
    fn find_sequence(
        &self,
        pattern: impl IntoIterator<Item = impl AsToken<TokenOf<GraphKindOf<Self>>>>,
    ) -> SearchResult
    {
        let iter = tokenizing_iter(pattern.into_iter());
        let pattern = self.graph().get_token_children(iter)?;
        self.find_ancestor(pattern)
    }
}

impl Searchable for &Hypergraph
{
    fn ctx(&self) -> SearchContext<Self>
    {
        SearchContext::new(self)
    }
}

impl Searchable for HypergraphRef
{
    fn ctx(&self) -> SearchContext<Self>
    {
        SearchContext::new(self.clone())
    }
}
