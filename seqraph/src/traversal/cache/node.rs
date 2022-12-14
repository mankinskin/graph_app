use crate::*;
use std::hash::Hash;

/// ordered according to priority
#[derive(Clone, Debug, Eq)]
pub struct WaitingNode<R: ResultKind + Eq, Q: TraversalQuery> {
    sub_index: usize,
    prev: CacheKey,
    // could be more efficient by referencing cache instead of storing path and query
    node: TraversalNode<R, Q>,
}
impl<R: ResultKind + Eq, Q: TraversalQuery> PartialEq for WaitingNode<R, Q> {
    fn eq(&self, other: &Self) -> bool {
        self.sub_index.eq(&other.sub_index)
    }
}
impl<R: ResultKind + Eq, Q: TraversalQuery> Ord for WaitingNode<R, Q> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.sub_index.partial_cmp(&other.sub_index)
            .unwrap_or(Ordering::Equal)
    }
}
impl<R: ResultKind + Eq, Q: TraversalQuery> PartialOrd for WaitingNode<R, Q> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.sub_index.partial_cmp(&other.sub_index).map(Ordering::reverse)
    }
}

//#[derive(Clone, Debug, Hash, PartialEq, Eq)]
//pub struct LocationNode {
//    location: ChildLocation,
//    token_pos: usize,
//}
//impl Wide for LocationNode {
//    fn width(&self) -> usize {
//        self.token_pos
//    }
//}
//impl From<StartLeaf> for LocationNode {
//    fn from(leaf: StartLeaf) -> Self {
//        Self {
//            location: leaf.entry(),
//            token_pos: leaf.width(),
//        }
//    }
//}
//impl GetCacheKey for LocationNode {
//    fn cache_key(&self) -> CacheKey {
//        CacheKey {
//            root: self.location.parent.index(),
//            token_pos: self.token_pos,
//        }
//    }
//}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct ParentNode<
    R: ResultKind,
    Q: TraversalQuery,
> {
    pub path: R::Primer,
    //pub location: LocationNode,
    pub query: Q,
    //pub num_patterns: usize,
    _ty: std::marker::PhantomData<(R, Q)>,
}
/// nodes generated during traversal.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TraversalNode<
    R: ResultKind,
    Q: TraversalQuery,
> {
    Start(usize, Q),
    /// at a parent.
    Parent(ParentNode<R, Q>),
    /// when the query has ended.
    QueryEnd(TraversalResult<R, Q>),
    /// at a position to be matched.
    /// (needed to enable breadth-first traversal)
    Child(PathPair<R::Advanced, Q>),
    /// at a match.
    Match(R::Advanced, Q),
    /// at a mismatch.
    Mismatch(TraversalResult<R, Q>),
    /// when a match was at the end of an index without parents.
    MatchEnd(R::Postfix, Q),
}
impl<
    R: ResultKind,
    Q: TraversalQuery,
> GetCacheKey for TraversalNode<R, Q> {
    fn cache_key(&self) -> CacheKey {
        match self {
            &TraversalNode::Start(root, _) => CacheKey {
                root,
                token_pos: 0,
            },
            TraversalNode::Parent(node) =>
                node.cache_key(),
            TraversalNode::Child(paths) =>
                paths.cache_key(),
            TraversalNode::Match(path, query) =>
                path.cache_key(),
            TraversalNode::MatchEnd(match_end, query) =>
                match_end.cache_key(),
            TraversalNode::Mismatch(found) =>
                found.cache_key(),
            TraversalNode::QueryEnd(found) =>
                found.cache_key(),
        }
    }
}
impl<
    R: ResultKind,
    Q: TraversalQuery,
> TraversalNode<R, Q> {
    pub fn query_node(index: usize, query: Q) -> Self {
        Self::Start(index, query)
    }
    pub fn match_node(path: R::Advanced, query: Q) -> Self {
        Self::Match(path, query)
    }
    pub fn child_node(paths: PathPair<R::Advanced, Q>) -> Self {
        Self::Child(paths)
    }
    pub fn parent_node(path: R::Primer, query: Q, num_patterns: usize) -> Self {
        Self::Parent(ParentNode {
            path,
            query,
            //num_patterns,
            _ty: Default::default(),
        })
    }
    pub fn query_end_node(found: TraversalResult<R, Q>) -> Self {
        Self::QueryEnd(found)
    }
    pub fn mismatch_node(found: TraversalResult<R, Q>) -> Self {
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