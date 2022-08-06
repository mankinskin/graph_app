use super::*;
use std::hash::Hash;

pub(crate) trait ToTraversalNode<
    Q: TraversalQuery,
>: Clone + Into<TraversalNode<Q>> {
    fn query_node(query: Q) -> Self;
    fn match_node(path: SearchPath, query: Q) -> Self;
    fn to_match_node(paths: PathPair<Q, SearchPath>) -> Self;
    fn parent_node(path: StartPath, query: Q) -> Self;
    fn query_end_node(found: Option<TraversalResult<SearchPath, Q>>) -> Self;
    fn mismatch_node(paths: PathPair<Q, SearchPath>) -> Self;
    fn match_end_node(match_end: MatchEnd, query: Q) -> Self;
    fn is_match(&self) -> bool;
}

/// nodes generated during traversal.
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub(crate) enum TraversalNode<
    Q: TraversalQuery,
> {
    /// a query is given.
    Query(Q),
    /// at a parent.
    Parent(StartPath, Q),
    /// when the query has ended.
    QueryEnd(Option<TraversalResult<SearchPath, Q>>),
    /// at a position to be matched.
    /// (needed to enable breadth-first traversal)
    ToMatch(PathPair<Q, SearchPath>),
    /// at a match.
    Match(SearchPath, Q),
    /// at a mismatch.
    Mismatch(PathPair<Q, SearchPath>),
    /// when a match was at the end of an index without parents.
    MatchEnd(MatchEnd, Q),
}
impl<
    Q: TraversalQuery,
> ToTraversalNode<Q> for TraversalNode<Q> {
    fn query_node(query: Q) -> Self {
        Self::Query(query)
    }
    fn match_node(path: SearchPath, query: Q) -> Self {
        Self::Match(path, query)
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
    fn is_match(&self) -> bool {
        matches!(self, TraversalNode::Match(_, _))
    }
}

pub(crate) type MatchNode = TraversalNode<QueryRangePath>;
pub(crate) type IndexingNode<Q> = TraversalNode<Q>;