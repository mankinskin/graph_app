use crate::*;
use std::hash::Hash;

/// ordered according to priority
//#[derive(Clone, Debug, Eq)]
//pub struct WaitingNode<R: ResultKind + Eq, Q: BaseQuery> {
//    sub_index: usize,
//    prev: CacheKey,
//    // could be more efficient by referencing cache instead of storing path and query
//    node: TraversalState<R, Q>,
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
pub struct StartState<
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
> StartState<R, Q> {
    pub fn new(index: Child, query: Q) -> Self {
        Self {
            index,
            query,
            _ty: Default::default(),
        }
    }
}
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct ParentState<
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
pub struct ChildState<
    R: ResultKind,
    Q: BaseQuery,
> {
    pub root: CacheKey,
    pub paths: PathPair<R::Advanced, Q>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EndState<
    R: ResultKind,
    Q: BaseQuery,
> {
    /// when the query has ended.
    QueryEnd(Option<ChildLocation>, CacheKey, TraversalResult<R, Q>),
    /// at a mismatch.
    Mismatch(Option<ChildLocation>, CacheKey, TraversalResult<R, Q>),
    /// when a match was at the end of an index without parents.
    MatchEnd(ChildLocation, R::Postfix, Q),
}
/// nodes generated during traversal.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TraversalState<
    R: ResultKind,
    Q: BaseQuery,
> {
    Start(StartState<R, Q>),
    /// at a parent.
    Parent(CacheKey, ParentState<R, Q>),
    /// at a position to be matched.
    /// (needed to enable breadth-first traversal)
    Child(CacheKey, ChildState<R, Q>),
    End(CacheKey, EndState<R, Q>),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NodeDirection {
    BottomUp,
    TopDown,
}
impl<
    R: ResultKind,
    Q: BaseQuery,
> EndState<R, Q> {
    pub fn entry_location(&self) -> Option<ChildLocation> {
        match self {
            Self::MatchEnd(entry, _, _) => Some(*entry),
            Self::QueryEnd(entry, _, _)
            | Self::Mismatch(entry, _, _) => *entry,
        }
    }
    pub fn node_direction(&self) -> NodeDirection {
        match self {
            Self::MatchEnd(_, _, _)
                => NodeDirection::BottomUp,
            Self::QueryEnd(_, _, _)
            | Self::Mismatch(_, _, _)
                => NodeDirection::TopDown,
        }
    }
    pub fn waiting_root_key(&self) -> Option<CacheKey> {
        match self {
            Self::MatchEnd(_, _, _)
                => None,
            Self::QueryEnd(_, root_key, _)
            | Self::Mismatch(_, root_key, _)
                => Some(*root_key),
        }
    }
}
impl<
    R: ResultKind,
    Q: BaseQuery,
> TraversalState<R, Q> {
    pub fn entry_location(&self) -> Option<ChildLocation> {
        match self {
            Self::Parent(_, state) => Some(state.path.root_child_location()),
            Self::Child(_, state) => Some(state.paths.get_path().role_path_child_location::<End>()),
            Self::End(_, state) => state.entry_location(),
            _ => None
        }
    }
    pub fn prev_key(&self) -> Option<CacheKey> {
        match self {
            Self::Parent(prev, _)
            | Self::Child(prev, _)
            | Self::End(prev, _)
                => Some(*prev),
            _ => None,
        }
    }
    pub fn node_direction(&self) -> NodeDirection {
        match self {
            Self::Parent(_, _)
            | Self::Start(_)
                => NodeDirection::BottomUp,
            Self::Child(_, _)
                => NodeDirection::TopDown,
            Self::End(_, state) => state.node_direction()
        }
    }
    pub fn is_bottom_up(&self) -> bool {
        self.node_direction() == NodeDirection::BottomUp
    }
    pub fn is_top_down(&self) -> bool {
        self.node_direction() == NodeDirection::TopDown
    }
}