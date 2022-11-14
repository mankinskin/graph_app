use crate::*;

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
    SingleIndex(Child),
    ParentMatchingPartially,
    UnknownKey,
    UnknownIndex,
    Unnecessary,
    EmptyRange,
}
//pub(crate) type QueryFound = TraversalResult<QueryRangePath>;
pub(crate) type SearchResult = Result<
    TraversalResult<
        <BaseResult as ResultKind>::Found,
        QueryRangePath
    >,
    NoMatch
>;

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
    pub async fn expect_pattern(
        &self,
        pattern: impl IntoIterator<Item = impl AsToken<T>>,
    ) -> Child {
        self.find_sequence(pattern).await.unwrap().unwrap_complete()
    }
    pub(crate) async fn find_ancestor(
        &self,
        pattern: impl IntoIterator<Item = impl Indexed>,
    ) -> SearchResult {
        let pattern = self.read().await.to_children(pattern);
        self.right_searcher().find_pattern_ancestor(pattern).await
    }
    #[allow(unused)]
    pub(crate) async fn find_parent(
        &self,
        pattern: impl IntoIterator<Item = impl Indexed>,
    ) -> SearchResult {
        let pattern = self.read().await.to_children(pattern);
        self.right_searcher().find_pattern_parent(pattern).await
    }
    pub(crate) async fn find_sequence(
        &self,
        pattern: impl IntoIterator<Item = impl AsToken<T>>,
    ) -> SearchResult {
        let iter = tokenizing_iter(pattern.into_iter());
        let pattern = self.read().await.to_token_children(iter)?;
        self.find_ancestor(pattern).await
    }
}