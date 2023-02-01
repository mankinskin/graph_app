use crate::*;
use std::hash::Hash;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InnerKind<
    R: ResultKind,
> {
    Parent(ParentState<R>),
    Child(ChildState<R>),
    End(EndState<R>),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WaitingState<
    R: ResultKind,
> {
    pub prev: CacheKey,
    pub matched: bool,
    pub state: ParentState<R>
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TraversalState<
    R: ResultKind,
> {
    pub prev: CacheKey,
    pub matched: bool,
    pub kind: InnerKind<R>
}
impl<
    R: ResultKind,
> From<WaitingState<R>> for TraversalState<R> {
    fn from(state: WaitingState<R>) -> Self {
        Self {
            prev: state.prev,
            matched: state.matched,
            kind: InnerKind::Parent(state.state),
        }
    }
}
impl<
    R: ResultKind,
> TraversalState<R> {
    pub fn entry_location(&self) -> Option<ChildLocation> {
        match &self.kind {
            InnerKind::Parent(state) => Some(state.path.root_child_location()),
            InnerKind::Child(state) => state.paths.get_path().role_leaf_child_location::<End>(),
            InnerKind::End(state) => state.entry_location(),
        }
    }
    pub fn prev_key(&self) -> CacheKey {
        self.prev
    }
    pub fn node_direction(&self) -> NodeDirection {
        match &self.kind {
            InnerKind::Parent(_)
                => NodeDirection::BottomUp,
            InnerKind::Child(_)
                => NodeDirection::TopDown,
            InnerKind::End(state) => state.node_direction()
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NodeDirection {
    BottomUp,
    TopDown,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct StartState<
    R: ResultKind,
> {
    pub index: Child,
    pub query: R::Query,
    pub _ty: std::marker::PhantomData<R>,
}
impl<
    R: ResultKind,
> StartState<R> {
    pub fn new(index: Child, query: R::Query) -> Self {
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
> {
    pub path: R::Primer,
    pub query: R::Query,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct ChildState<
    R: ResultKind,
> {
    pub root: CacheKey,
    pub paths: PathPair<R::Advanced, R::Query>,
}

// End types:
// - top down match-mismatch
// - top down match-query end
// - bottom up-no matching parents
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RangeKind {
    /// when the query has ended.
    QueryEnd,
    /// at a mismatch.
    Mismatch,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EndKind<R: ResultKind> {
    Range(RangeEnd<R>),
    Postfix(PostfixEnd<R>),
    Complete(Child),
}
impl<R: ResultKind> From<MatchEnd<RootedRolePath<Start>>> for EndKind<R> {
    fn from(postfix: MatchEnd<RootedRolePath<Start>>) -> Self {
        match postfix {
            MatchEnd::Complete(c) => EndKind::Complete(c),
            MatchEnd::Path(path) => EndKind::Postfix(PostfixEnd {
                path: path.into(),
            }),
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RangeEnd<R: ResultKind> {
    pub entry: ChildLocation,
    pub kind: RangeKind,
    pub path: R::Advanced,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PostfixEnd<R: ResultKind> {
    pub path: R::Primer,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EndState<
    R: ResultKind,
> {
    pub root: CacheKey,
    pub kind: EndKind<R>,
    pub query: R::Query,
}

impl<
    R: ResultKind,
> EndState<R> {
    pub fn entry_location(&self) -> Option<ChildLocation> {
        match &self.kind {
            EndKind::Range(state) => Some(state.entry),
            EndKind::Postfix(_) => None,
            EndKind::Complete(_) => None,
        }
    }
    pub fn node_direction(&self) -> NodeDirection {
        match self.kind {
            EndKind::Range(_) => NodeDirection::TopDown,
            EndKind::Postfix(_) => NodeDirection::BottomUp,
            EndKind::Complete(_) => NodeDirection::BottomUp,
        }
    }
    pub fn waiting_root_key(&self) -> Option<CacheKey> {
        match self.kind {
            EndKind::Range(_) => Some(self.root),
            EndKind::Postfix(_) => None,
            EndKind::Complete(_) => None,
        }
    }
    pub fn is_complete(&self) -> bool {
        matches!(self.kind, EndKind::Complete(_))
    }
}
//impl<R: ResultKind> PartialOrd for EndKind<R> {
//    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
//        match (self, other) {
//            (Self::Complete(l), Self::Complete(r)) =>
//                l.width().partial_cmp(&r.width()),
//            // complete always greater than prefix/postfix/range
//            (Self::Complete(_), _) => Some(Ordering::Greater),
//            (_, Self::Complete(_)) => Some(Ordering::Less),
//            (Self::Range(l), Self::Range(r)) =>
//                l.path.partial_cmp(&r.path),
//        }
//    }
//}
//impl<R: ResultKind> Ord for EndKind<R> {
//    fn cmp(&self, other: &Self) -> Ordering {
//        self.partial_cmp(&other)
//            .unwrap_or(Ordering::Equal)
//    }
//}