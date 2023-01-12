use crate::*;
use std::hash::Hash;

/// ordered according to priority
//#[derive(Clone, Debug, Eq)]
//pub struct WaitingNode<R: ResultKind + Eq, Q: BaseQuery> {
//    sub_index: usize,
//    prev: CacheKey,
//    // could be more efficient by referencing cache instead of storing path and query
//    node: TraversalNode<R, Q>,
//}
//impl<R: ResultKind + Eq, Q: BaseQuery> PartialEq for WaitingNode<R, Q> {
//    fn eq(&self, other: &Self) -> bool {
//        self.sub_index.eq(&other.sub_index)
//    }
//}
//impl<R: ResultKind + Eq, Q: BaseQuery> Ord for WaitingNode<R, Q> {
//    fn cmp(&self, other: &Self) -> Ordering {
//        self.sub_index.partial_cmp(&other.sub_index)
//            .unwrap_or(Ordering::Equal)
//    }
//}
//impl<R: ResultKind + Eq, Q: BaseQuery> PartialOrd for WaitingNode<R, Q> {
//    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
//        self.sub_index.partial_cmp(&other.sub_index).map(Ordering::reverse)
//    }
//}

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
//impl From<PathLeaf> for LocationNode {
//    fn from(leaf: PathLeaf) -> Self {
//        Self {
//            location: leaf.child_location(),
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
pub struct StartNode<
    R: ResultKind,
    Q: BaseQuery,
> {
    pub index: Child,
    pub query: Q,
    pub _ty: std::marker::PhantomData<(R, Q)>,
}
impl<
    R: ResultKind,
    Q: BaseQuery,
> StartNode<R, Q> {
    pub fn new(index: Child, query: Q) -> Self {
        Self {
            index,
            query,
            _ty: Default::default(),
        }
    }
}
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct ParentNode<
    R: ResultKind,
    Q: BaseQuery,
> {
    pub path: R::Primer,
    //pub location: LocationNode,
    pub query: Q,
    //pub num_patterns: usize,
    pub _ty: std::marker::PhantomData<(R, Q)>,
}
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct ChildNode<
    R: ResultKind,
    Q: BaseQuery,
> {
    pub root: CacheKey,
    pub paths: PathPair<R::Advanced, Q>,
}
/// nodes generated during traversal.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TraversalNode<
    R: ResultKind,
    Q: BaseQuery,
> {
    Start(StartNode<R, Q>),
    /// at a parent.
    Parent(CacheKey, ParentNode<R, Q>),
    /// at a position to be matched.
    /// (needed to enable breadth-first traversal)
    Child(CacheKey, ChildNode<R, Q>),
    /// when the query has ended.
    QueryEnd(CacheKey, Option<ChildLocation>, TraversalResult<R, Q>),
    /// at a mismatch.
    Mismatch(CacheKey, Option<ChildLocation>, TraversalResult<R, Q>),
    /// when a match was at the end of an index without parents.
    MatchEnd(CacheKey, ChildLocation, R::Postfix, Q),
    ///// at a match.
    //Match(R::Advanced, Q),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NodeDirection {
    BottomUp,
    TopDown,
}
impl<
    R: ResultKind,
    Q: BaseQuery,
> TraversalNode<R, Q> {
    //pub fn query_node(index: usize, query: Q) -> Self {
    //    Self::Start(index, query)
    //}
    //pub fn match_node(path: R::Advanced, query: Q) -> Self {
    //    Self::Match(path, query)
    //}
    //pub fn child_node(prev: CacheKey, root: CacheKey, paths: PathPair<R::Advanced, Q>) -> Self {
    //    Self::Child(ChildNode {
    //        root,
    //        paths
    //    })
    //}
    //pub fn parent_node(path: R::Primer, query: Q, num_patterns: usize) -> Self {
    //    Self::Parent(ParentNode {
    //        path,
    //        query,
    //        //num_patterns,
    //        _ty: Default::default(),
    //    })
    //}
    //pub fn query_end_node(found: TraversalResult<R, Q>) -> Self {
    //    Self::QueryEnd(found)
    //}
    //pub fn mismatch_node(found: TraversalResult<R, Q>) -> Self {
    //    Self::Mismatch(found)
    //}
    pub fn entry_location(&self) -> Option<ChildLocation> {
        match self {
            Self::Parent(_, node) => Some(node.path.root_child_location()),
            Self::Child(_, node) => Some(node.paths.get_path().role_path_child_location::<End>()),
            Self::MatchEnd(_, entry, _, _) => Some(*entry),
            Self::QueryEnd(_, entry, _)
            | Self::Mismatch(_, entry, _) => *entry,
            _ => None
        }
    }
    pub fn prev_key(&self) -> Option<CacheKey> {
        match self {
            Self::Parent(prev, _)
            | Self::Child(prev, _)
            | Self::Mismatch(prev, _, _)
            | Self::QueryEnd(prev, _, _)
            | Self::MatchEnd(prev, _, _, _)
                => Some(*prev),
            _ => None,
        }
    }
    pub fn node_direction(&self) -> NodeDirection {
        match self {
            Self::Parent(_, _)
            | Self::Start(_)
            | Self::MatchEnd(_, _, _, _)
                => NodeDirection::BottomUp,
            Self::Child(_, _)
            | Self::Mismatch(_, _, _)
            | Self::QueryEnd(_, _, _)
                => NodeDirection::TopDown,
        }
    }
    pub fn is_bottom_up(&self) -> bool {
        self.node_direction() == NodeDirection::BottomUp
    }
    pub fn is_top_down(&self) -> bool {
        self.node_direction() == NodeDirection::TopDown
    }
    //pub fn match_end_node(match_end: R::Postfix, query: Q) -> Self {
    //    Self::MatchEnd(match_end, query)
    //}
    //#[allow(unused)]
    //pub fn is_match(&self) -> bool {
    //    matches!(self, TraversalNode::Match(_, _))
    //}
    //pub fn get_parent_path(&self) -> Option<&R::Primer> {
    //    match self {
    //        TraversalNode::Parent(path, _) => Some(path),
    //        _ => None
    //    }
    //}
}