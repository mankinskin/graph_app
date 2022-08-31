use super::*;
use std::hash::Hash;

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
    QueryEnd(TraversalResult<Q>),
    /// at a position to be matched.
    /// (needed to enable breadth-first traversal)
    ToMatch(PathPair<Q>),
    /// at a match.
    Match(SearchPath, Q),
    /// at a mismatch.
    Mismatch(TraversalResult<Q>),
    /// when a match was at the end of an index without parents.
    MatchEnd(MatchEnd, Q),
}
impl<
    Q: TraversalQuery,
> TraversalNode<Q> {
    pub fn query_node(query: Q) -> Self {
        Self::Query(query)
    }
    pub fn match_node(path: SearchPath, query: Q) -> Self {
        Self::Match(path, query)
    }
    pub fn to_match_node(paths: PathPair<Q>) -> Self {
        Self::ToMatch(paths)
    }
    pub fn parent_node(path: StartPath, query: Q) -> Self {
        Self::Parent(path, query)
    }
    pub fn query_end_node(found: TraversalResult<Q>) -> Self {
        Self::QueryEnd(found)
    }
    pub fn mismatch_node(found: TraversalResult<Q>) -> Self {
        Self::Mismatch(found)
    }
    pub fn match_end_node(match_end: MatchEnd, query: Q) -> Self {
        Self::MatchEnd(match_end, query)
    }
    #[allow(unused)]
    pub fn is_match(&self) -> bool {
        matches!(self, TraversalNode::Match(_, _))
    }
    pub fn get_parent_path(&self) -> Option<&StartPath> {
        match self {
            TraversalNode::Parent(path, _) => Some(path),
            _ => None
        }
    }
}
pub(crate) type MatchNode = TraversalNode<QueryRangePath>;
pub(crate) type IndexingNode<Q> = TraversalNode<Q>;