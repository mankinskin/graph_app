mod searcher;

#[macro_use]
#[cfg(test)]
pub mod tests;

use crate::{
    graph::{
        kind::TokenOf,
        HypergraphRef,
    },
    traversal::traversable::{
        GraphKindOf,
        Traversable,
    },
    vertex::{
        child::Child,
        pattern::Pattern,
        token::{
            tokenizing_iter,
            AsToken,
        },
        PatternId,
    },
};
pub use searcher::*;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum NoMatch {
    EmptyPatterns,
    NoParents,
    NoChildPatterns,
    NotFound,
    NoMatchingParent(crate::vertex::VertexIndex),
    InvalidPattern(PatternId),
    InvalidChild(usize),
    InvalidPatternRange(PatternId, Pattern, String),
    SingleIndex(Child),
    ParentMatchingPartially,
    UnknownKey,
    UnknownIndex,
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
        pattern: impl IntoIterator<Item = impl crate::vertex::indexed::Indexed>,
    ) -> SearchResult {
        let pattern = self.graph().to_children(pattern);
        self.searcher().find_pattern_ancestor(pattern)
    }
    fn find_parent(
        &self,
        pattern: impl IntoIterator<Item = impl crate::vertex::indexed::Indexed>,
    ) -> SearchResult {
        let pattern = self.graph().to_children(pattern);
        self.searcher().find_pattern_parent(pattern)
    }
    fn find_sequence(
        &self,
        pattern: impl IntoIterator<Item = impl AsToken<TokenOf<GraphKindOf<Self>>>>,
    ) -> SearchResult {
        let iter = tokenizing_iter(pattern.into_iter());
        let pattern = self.graph().to_token_children(iter)?;
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
