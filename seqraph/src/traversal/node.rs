use super::*;
use std::hash::Hash;

pub(crate) trait ToTraversalNode<
    Q: TraversalQuery,
>: Clone + Into<TraversalNode<Q>> {
    fn query_node(query: Q) -> Self;
    fn match_node(path: SearchPath, query: Q, old_query: Q) -> Self;
    fn to_match_node(paths: PathPair<Q, SearchPath>) -> Self;
    fn parent_node(path: StartPath, query: Q) -> Self;
    fn query_end_node(found: Option<TraversalResult<SearchPath, Q>>) -> Self;
    fn mismatch_node(paths: PathPair<Q, SearchPath>) -> Self;
    fn match_end_node(match_end: MatchEnd, query: Q) -> Self;
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub(crate) enum TraversalNode<
    Q: TraversalQuery,
> {
    Query(Q),
    Parent(StartPath, Q),
    QueryEnd(Option<TraversalResult<SearchPath, Q>>),
    ToMatch(PathPair<Q, SearchPath>),
    Match(SearchPath, Q, Q),
    Mismatch(PathPair<Q, SearchPath>),
    MatchEnd(MatchEnd, Q),
}
impl<
    Q: TraversalQuery,
> ToTraversalNode<Q> for TraversalNode<Q> {
    fn query_node(query: Q) -> Self {
        Self::Query(query)
    }
    fn match_node(path: SearchPath, query: Q, old_query: Q) -> Self {
        Self::Match(path, query, old_query)
    }
    fn to_match_node(paths: PathPair<Q, SearchPath>) -> Self {
        Self::ToMatch(paths)
    }
    fn parent_node(path: StartPath, query: Q) -> Self {
        Self::Parent(path, query)
    }
    fn query_end_node(found: Option<TraversalResult<SearchPath, Q>>) -> Self {
        Self::QueryEnd(found)
    }
    fn mismatch_node(paths: PathPair<Q, SearchPath>) -> Self {
        Self::Mismatch(paths)
    }
    fn match_end_node(match_end: MatchEnd, query: Q) -> Self {
        Self::MatchEnd(match_end, query)
    }
}

pub(crate) type MatchNode = TraversalNode<QueryRangePath>;
pub(crate) type IndexingNode<Q> = TraversalNode<Q>;