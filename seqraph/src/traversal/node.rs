use super::*;

pub(crate) trait ToTraversalNode<
    Q: TraversalQuery,
>: Clone + Into<TraversalNode<Q>> {
    fn query_node(query: Q) -> Self;
    fn match_node(path: SearchPath, query: Q, old_query: Q) -> Self;
    fn to_match_node(paths: PathPair<Q, SearchPath>) -> Self;
    fn parent_node(path: StartPath, query: Q) -> Self;
    fn end_node(found: Option<TraversalResult<SearchPath, Q>>) -> Self;
    fn mismatch_node(paths: PathPair<Q, SearchPath>) -> Self;
}

#[derive(Clone, Debug)]
pub(crate) enum TraversalNode<
    Q: TraversalQuery,
> {
    Query(Q),
    Parent(StartPath, Q),
    End(Option<TraversalResult<SearchPath, Q>>),
    ToMatch(PathPair<Q, SearchPath>),
    Match(SearchPath, Q, Q),
    Mismatch(PathPair<Q, SearchPath>),
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
    fn end_node(found: Option<TraversalResult<SearchPath, Q>>) -> Self {
        Self::End(found)
    }
    fn mismatch_node(paths: PathPair<Q, SearchPath>) -> Self {
        Self::Mismatch(paths)
    }
}

pub(crate) type MatchNode = TraversalNode<QueryRangePath>;
pub(crate) type IndexingNode<Q> = TraversalNode<Q>;