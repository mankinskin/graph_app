use crate::{
    QueryResult,
};
use super::*;

pub(crate) trait ToTraversalNode<
    Q: TraversalQuery,
    G: TraversalPath,
    >: Clone + Into<TraversalNode<Q, G>> {
    fn query_node(query: Q) -> Self;
    fn match_node(path: G, query: Q, old_query: Q) -> Self;
    fn to_match_node(paths: PathPair<Q, G>) -> Self;
    fn parent_node(path: StartPath, query: Q) -> Self;
    fn end_node(found: Option<QueryResult<Q>>) -> Self;
    fn mismatch_node(paths: PathPair<Q, G>) -> Self;
}

#[derive(Clone, Debug)]
pub(crate) enum TraversalNode<
    Q: TraversalQuery,
    G: TraversalPath,
> {
    Query(Q),
    Parent(StartPath, Q),
    End(Option<QueryResult<Q>>),
    ToMatch(PathPair<Q, G>),
    Match(G, Q, Q),
    Mismatch(PathPair<Q, G>),
}
impl<
    Q: TraversalQuery,
    G: TraversalPath,
> ToTraversalNode<Q, G> for TraversalNode<Q, G> {
    fn query_node(query: Q) -> Self {
        Self::Query(query)
    }
    fn match_node(path: G, query: Q, old_query: Q) -> Self {
        Self::Match(path, query, old_query)
    }
    fn to_match_node(paths: PathPair<Q, G>) -> Self {
        Self::ToMatch(paths)
    }
    fn parent_node(path: StartPath, query: Q) -> Self {
        Self::Parent(path, query)
    }
    fn end_node(found: Option<QueryResult<Q>>) -> Self {
        Self::End(found)
    }
    fn mismatch_node(paths: PathPair<Q, G>) -> Self {
        Self::Mismatch(paths)
    }
}

pub(crate) type MatchNode = TraversalNode<QueryRangePath, GraphRangePath>;
pub(crate) type IndexingNode<Q> = TraversalNode<Q, GraphRangePath>;