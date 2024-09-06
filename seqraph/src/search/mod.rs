use searcher::*;

use crate::{
    graph::{
        HypergraphRef,
        kind::TokenOf,
    },
    traversal::traversable::{
        GraphKindOf,
        Traversable,
    },
};
use crate::graph::vertex::{child::Child, pattern::Pattern, token::{
    AsToken,
    tokenizing_iter,
}, VertexIndex};
use crate::graph::vertex::pattern::id::PatternId;

pub mod searcher;

#[macro_use]
#[cfg(test)]
pub mod tests;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum NoMatch {
    EmptyPatterns,
    NoParents,
    NoChildPatterns,
    NotFound,
    NoMatchingParent(VertexIndex),
    InvalidPattern(PatternId),
    InvalidChild(usize),
    InvalidPatternRange(PatternId, Pattern, String),
    SingleIndex(Child),
    ParentMatchingPartially,
    UnknownKey,
    UnknownIndex,
    UnknownToken,
    Unnecessary,
    EmptyRange,
}

pub trait Searchable: Traversable {
    fn searcher(&self) -> Searcher<Self>;
    //pub fn expect_pattern(
    //    &self,
    //    pattern: impl IntoIterator<Item = impl AsToken<T>>,
    //) -> Child {
    //    self.find_sequence(pattern).unwrap().unwrap_complete()
    //}
    fn find_ancestor(
        &self,
        pattern: impl IntoIterator<Item = impl crate::graph::vertex::has_vertex_index::HasVertexIndex>,
    ) -> SearchResult {
        let pattern = self.graph().to_children(pattern);
        self.searcher().find_pattern_ancestor(pattern)
    }
    fn find_parent(
        &self,
        pattern: impl IntoIterator<Item = impl crate::graph::vertex::has_vertex_index::HasVertexIndex>,
    ) -> SearchResult {
        let pattern = self.graph().to_children(pattern);
        self.searcher().find_pattern_parent(pattern)
    }
    fn find_sequence(
        &self,
        pattern: impl IntoIterator<Item = impl AsToken<TokenOf<GraphKindOf<Self>>>>,
    ) -> SearchResult {
        let iter = tokenizing_iter(pattern.into_iter());
        let pattern = self.graph().get_token_children(iter)?;
        self.find_ancestor(pattern)
    }
}

impl<'g> Searchable for &'g crate::graph::Hypergraph {
    fn searcher(&self) -> Searcher<Self> {
        Searcher::new(self)
    }
}

impl Searchable for HypergraphRef {
    fn searcher(&self) -> Searcher<Self> {
        Searcher::new(self.clone())
    }
}
