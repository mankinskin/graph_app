use crate::*;

mod searcher;

#[macro_use]
#[cfg(test)]
pub mod tests;

pub use searcher::*;

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
    Unnecessary,
    EmptyRange,
}

impl<'t, 'g> HypergraphRef
{
    pub fn searcher(&'g self) -> Searcher {
        Searcher::new(self.clone())
    }
    //pub fn expect_pattern(
    //    &self,
    //    pattern: impl IntoIterator<Item = impl AsToken<T>>,
    //) -> Child {
    //    self.find_sequence(pattern).unwrap().unwrap_complete()
    //}
    pub fn find_ancestor(
        &self,
        pattern: impl IntoIterator<Item = impl Indexed>,
    ) -> SearchResult {
        let pattern = self.graph().to_children(pattern);
        self.searcher().find_pattern_ancestor(pattern)
    }
    #[allow(unused)]
    pub fn find_parent(
        &self,
        pattern: impl IntoIterator<Item = impl Indexed>,
    ) -> SearchResult {
        let pattern = self.graph().to_children(pattern);
        self.searcher().find_pattern_parent(pattern)
    }
    pub fn find_sequence(
        &self,
        pattern: impl IntoIterator<Item = impl AsToken<<BaseGraphKind as GraphKind>::Token>>,
    ) -> SearchResult {
        let iter = tokenizing_iter(pattern.into_iter());
        let pattern = self.graph().to_token_children(iter)?;
        self.find_ancestor(pattern)
    }
}