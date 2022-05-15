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
}

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
impl ResultOrd for GraphRangePath {
    fn is_complete(&self) -> bool {
        false
    }
}
impl ResultOrd for FoundPath {
    fn is_complete(&self) -> bool {
        matches!(self, FoundPath::Complete(_))
    }
}
impl Wide for FoundPath {
    fn width(&self) -> usize {
        match self {
            Self::Complete(c) => c.width,
            Self::Range(r) => r.width(),
        }
    }
}
impl<Rhs: ResultOrd> PartialOrd<Rhs> for FoundPath {
    fn partial_cmp(&self, other: &Rhs) -> Option<Ordering> {
        Some(ResultOrd::cmp(self, other))
    }
}
impl<Rhs: ResultOrd> PartialEq<Rhs> for FoundPath {
    fn eq(&self, other: &Rhs) -> bool {
        ResultOrd::eq(self, other)
    }
}
impl<Rhs: ResultOrd> PartialOrd<Rhs> for GraphRangePath {
    fn partial_cmp(&self, other: &Rhs) -> Option<Ordering> {
        Some(ResultOrd::cmp(self, other))
    }
}
impl<Rhs: ResultOrd> PartialEq<Rhs> for GraphRangePath {
    fn eq(&self, other: &Rhs) -> bool {
        ResultOrd::eq(self, other)
    }
}
#[derive(Debug, Clone, Eq)]
pub(crate) enum FoundPath {
    Complete(Child),
    Range(GraphRangePath),
}
impl<
    'a: 'g,
    'g,
> FoundPath {
    pub(crate) fn new<
        T: Tokenize + 'a,
        D: MatchDirection + 'a,
        Trav: Traversable<'a, 'g, T>,
    >(trav: &'a Trav, range_path: GraphRangePath) -> Self {
        if range_path.is_complete::<_, D, _>(trav) {
            Self::Complete(Into::<StartPath>::into(range_path).entry().parent)
        } else {
            Self::Range(range_path)
        }
    }
    #[track_caller]
    pub fn unwrap_complete(self) -> Child {
        match self {
            Self::Complete(index) => index,
            _ => panic!("Unable to unwrap {:?} as complete.", self),
        }
    }
    #[track_caller]
    pub fn expect_complete(self, msg: &str) -> Child {
        match self {
            Self::Complete(index) => index,
            _ => panic!("Unable to unwrap {:?} as complete: {}", self, msg),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueryResult<Q: TraversalQuery> {
    pub(crate) found: FoundPath,
    pub(crate) query: Q,
}

impl<Q: TraversalQuery> QueryResult<Q> {
    pub(crate) fn new(found: impl Into<FoundPath>, query: Q) -> Self {
        Self {
            found: found.into(),
            query,
        }
    }
    #[track_caller]
    pub fn unwrap_complete(self) -> Child {
        self.found.unwrap_complete()
    }
    #[track_caller]
    pub fn expect_complete(self, msg: &str) -> Child {
        self.found.expect_complete(msg)
    }
}
impl<Q: QueryPath> QueryResult<Q> {
    pub fn complete(query: impl IntoPattern, index: impl AsChild) -> Self {
        Self {
            found: FoundPath::Complete(index.as_child()),
            query: Q::complete(query),
        }
    }
}
pub type QueryFound = QueryResult<QueryRangePath>;
pub type SearchResult = Result<QueryFound, NoMatch>;

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
    pub fn find_parent(
        &self,
        pattern: impl IntoIterator<Item = impl Indexed>,
    ) -> SearchResult {
        let pattern = self.read().unwrap().to_children(pattern);
        self.right_searcher().find_pattern_parent(pattern)
    }
    pub fn find_sequence(
        &self,
        pattern: impl IntoIterator<Item = impl AsToken<T>>,
    ) -> SearchResult {
        let iter = tokenizing_iter(pattern.into_iter());
        let pattern = self.read().unwrap().to_token_children(iter)?;
        self.find_ancestor(pattern)
    }
}