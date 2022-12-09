use super::*;
use std::hash::Hash;

//pub type NodeTraversalResult<R, Q> =
//    TraversalResult<<R as ResultKind>::Found, Q>;

/// nodes generated during traversal.
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum TraversalNode<
    R: ResultKind,
    Q: TraversalQuery,
> {
    /// a query is given.
    Query(Q),
    /// at a parent.
    Parent(ParentNode<R, Q>),
    /// when the query has ended.
    QueryEnd(TraversalResult<<R as ResultKind>::Found, Q>),
    /// at a position to be matched.
    /// (needed to enable breadth-first traversal)
    ToMatch(PathPair<R::Advanced, Q>),
    /// at a match.
    Match(R::Advanced, Q),
    /// at a mismatch.
    Mismatch(TraversalResult<<R as ResultKind>::Found, Q>),
    /// when a match was at the end of an index without parents.
    MatchEnd(R::Postfix, Q),
}
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct ParentNode<
    R: ResultKind,
    Q: TraversalQuery,
> {
    pub path: R::Primer,
    pub query: Q,
    pub num_patterns: usize,
}
impl<
    R: ResultKind,
    Q: TraversalQuery,
> TraversalNode<R, Q> {
    pub fn query_node(query: Q) -> Self {
        Self::Query(query)
    }
    pub fn match_node(path: R::Advanced, query: Q) -> Self {
        Self::Match(path, query)
    }
    pub fn to_match_node(paths: PathPair<R::Advanced, Q>) -> Self {
        Self::ToMatch(paths)
    }
    pub fn parent_node(path: R::Primer, query: Q, num_patterns: usize) -> Self {
        Self::Parent(path, query, num_patterns)
    }
    pub fn query_end_node(found: TraversalResult<<R as ResultKind>::Found, Q>) -> Self {
        Self::QueryEnd(found)
    }
    pub fn mismatch_node(found: TraversalResult<<R as ResultKind>::Found, Q>) -> Self {
        Self::Mismatch(found)
    }
    pub fn match_end_node(match_end: R::Postfix, query: Q) -> Self {
        Self::MatchEnd(match_end, query)
    }
    #[allow(unused)]
    pub fn is_match(&self) -> bool {
        matches!(self, TraversalNode::Match(_, _))
    }
    //pub fn get_parent_path(&self) -> Option<&R::Primer> {
    //    match self {
    //        TraversalNode::Parent(path, _) => Some(path),
    //        _ => None
    //    }
    //}
}
//pub type MatchNode = TraversalNode<MatchEnd, QueryRangePath>;
//pub type IndexingNode<Q> = TraversalNode<MatchEnd, Q>;