use searcher::*;

use crate::{
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

pub mod searcher;

#[macro_use]
#[cfg(test)]
pub mod tests;



pub trait Searchable: Traversable
{
    fn searcher(&self) -> Searcher<Self>;
    //pub fn expect_pattern(
    //    &self,
    //    pattern: impl IntoIterator<Item = impl AsToken<T>>,
    //) -> Child {
    //    self.find_sequence(pattern).unwrap().unwrap_complete()
    //}
    fn find_ancestor(
        &self,
        pattern: impl IntoIterator<
            Item = impl crate::graph::vertex::has_vertex_index::HasVertexIndex,
        >,
    ) -> SearchResult
    {
        let pattern = self.graph().to_children(pattern);
        self.searcher().find_pattern_ancestor(pattern)
    }
    fn find_parent(
        &self,
        pattern: impl IntoIterator<
            Item = impl crate::graph::vertex::has_vertex_index::HasVertexIndex,
        >,
    ) -> SearchResult
    {
        let pattern = self.graph().to_children(pattern);
        self.searcher().find_pattern_parent(pattern)
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
    fn searcher(&self) -> Searcher<Self>
    {
        Searcher::new(self)
    }
}

impl Searchable for HypergraphRef
{
    fn searcher(&self) -> Searcher<Self>
    {
        Searcher::new(self.clone())
    }
}
