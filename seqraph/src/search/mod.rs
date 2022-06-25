use std::cmp::Ordering;

use crate::{
    vertex::*,
    *,
};
mod searcher;
mod match_direction;
#[macro_use]
#[cfg(test)]
pub(crate) mod tests;

pub use searcher::*;
pub use match_direction::*;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum NoMatch {
    EmptyPatterns,
    NoParents,
    NoChildPatterns,
    NotFound,
    NoMatchingParent(VertexIndex),
    InvalidPattern(PatternId),
    InvalidPatternRange(PatternId, Pattern, String),
    SingleIndex,
    ParentMatchingPartially,
    UnknownKey,
    UnknownIndex,
    Unnecessary,
    EmptyRange,
}
pub(crate) type SearchFoundPath = FoundPath<SearchPath>;

pub trait ResultOrd: Wide {
    fn is_complete(&self) -> bool;
    fn cmp(&self, other: impl ResultOrd) -> Ordering {
        let l = self.is_complete();
        let r = other.is_complete();
        if l == r {
            self.width().cmp(&other.width())
        } else {
            l.cmp(&r)
        }
    }
    fn eq(&self, other: impl ResultOrd) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}
impl<T: ResultOrd> ResultOrd for &T {
    fn is_complete(&self) -> bool {
        ResultOrd::is_complete(*self)
    }
}
impl ResultOrd for SearchPath {
    fn is_complete(&self) -> bool {
        false
    }
}
impl<Rhs: ResultOrd> PartialOrd<Rhs> for SearchPath {
    fn partial_cmp(&self, other: &Rhs) -> Option<Ordering> {
        Some(ResultOrd::cmp(self, other))
    }
}
impl<Rhs: ResultOrd> PartialEq<Rhs> for SearchPath {
    fn eq(&self, other: &Rhs) -> bool {
        ResultOrd::eq(self, other)
    }
}
pub(crate) type QueryFound = TraversalResult<SearchPath, QueryRangePath>;
pub(crate) type SearchResult = Result<QueryFound, NoMatch>;

impl<'t, 'g, T> HypergraphRef<T>
where
    T: Tokenize + 't,
{
    pub(crate) fn searcher<D: MatchDirection>(&'g self) -> Searcher<T, D> {
        Searcher::new(self.clone())
    }
    pub(crate) fn right_searcher(&'g self) -> Searcher<T, Right> {
        self.searcher()
    }
    pub fn left_searcher(&'g self) -> Searcher<T, Left> {
        self.searcher()
    }
    pub fn expect_pattern(
        &self,
        pattern: impl IntoIterator<Item = impl AsToken<T>>,
    ) -> Child {
        self.find_sequence(pattern).unwrap().unwrap_complete()
    }
    pub(crate) fn find_ancestor(
        &self,
        pattern: impl IntoIterator<Item = impl Indexed>,
    ) -> SearchResult {
        let pattern = self.read().unwrap().to_children(pattern);
        self.right_searcher().find_pattern_ancestor(pattern)
    }
    #[allow(unused)]
    pub(crate) fn find_parent(
        &self,
        pattern: impl IntoIterator<Item = impl Indexed>,
    ) -> SearchResult {
        let pattern = self.read().unwrap().to_children(pattern);
        self.right_searcher().find_pattern_parent(pattern)
    }
    pub(crate) fn find_sequence(
        &self,
        pattern: impl IntoIterator<Item = impl AsToken<T>>,
    ) -> SearchResult {
        let iter = tokenizing_iter(pattern.into_iter());
        let pattern = self.read().unwrap().to_token_children(iter)?;
        self.find_ancestor(pattern)
    }
}