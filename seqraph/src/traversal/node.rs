use crate::{
    QueryResult, ChildLocation,
};
use super::*;

pub(crate) trait ToTraversalNode<
    Q: TraversalQuery,
    G: TraversalPath,
    >: Clone + Into<TraversalNode<Q, G>> {
    fn query_node(query: Q) -> Self;
    fn match_node(path: G, query: Q, old_query: Q) -> Self;
    fn root_node(path: Option<StartPath>, query: Q, parent_entry: ChildLocation) -> Self;
    fn end_node(found: Option<QueryResult<Q>>) -> Self;
    fn mismatch_node(paths: PathPair<Q, G>) -> Self;
}

#[derive(Clone, Debug)]
pub(crate) enum TraversalNode<
    Q: TraversalQuery,
    G: TraversalPath,
> {
    Query(Q),
    Match(G, Q, Q),
    Root(Option<StartPath>, Q, ChildLocation),
    End(Option<QueryResult<Q>>),
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
    fn root_node(path: Option<StartPath>, query: Q, parent_entry: ChildLocation) -> Self {
        Self::Root(path, query, parent_entry)
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