
use crate::*;

// End types:
// - top down match-mismatch
// - top down match-query end
// - bottom up-no matching parents
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RangeEnd {
    pub kind: RangeKind,
    pub path: SearchPath,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EndState {
    pub root_pos: TokenLocation,
    pub kind: EndKind,
    pub query: QueryState,
}

impl EndState {
    pub fn entry_location(&self) -> Option<ChildLocation> {
        match &self.kind {
            EndKind::Range(state) => Some(GraphRootChild::<Start>::root_child_location(&state.path)),
            EndKind::Postfix(_) => None,
            EndKind::Prefix(_) => None,
            EndKind::Complete(_) => None,
        }
    }
    pub fn node_direction(&self) -> NodeDirection {
        match self.kind {
            EndKind::Range(_) => NodeDirection::TopDown,
            EndKind::Postfix(_) => NodeDirection::BottomUp,
            EndKind::Prefix(_) => NodeDirection::TopDown,
            EndKind::Complete(_) => NodeDirection::BottomUp,
        }
    }
    pub fn waiting_root_key(&self) -> Option<CacheKey> {
        match &self.kind {
            EndKind::Range(_) => Some(self.root_key()),
            EndKind::Postfix(_) => None,
            EndKind::Prefix(_) => None,
            EndKind::Complete(_) => None,
        }
    }
    pub fn is_complete(&self) -> bool {
        matches!(self.kind, EndKind::Complete(_))
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EndKind {
    Range(RangeEnd),
    Postfix(Primer),
    Prefix(RootedRolePath<End>),
    Complete(Child),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RangeKind {
    /// when the query has ended.
    QueryEnd,
    /// at a mismatch.
    Mismatch,
}
impl From<MatchEnd<RootedRolePath<Start>>> for EndKind {
    fn from(postfix: MatchEnd<RootedRolePath<Start>>) -> Self {
        match postfix {
            MatchEnd::Complete(c) => EndKind::Complete(c),
            MatchEnd::Path(path) => EndKind::Postfix(
                path.into(),
            ),
        }
    }
}
//impl PartialOrd for EndKind {
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
//impl Ord for EndKind {
//    fn cmp(&self, other: &Self) -> Ordering {
//        self.partial_cmp(&other)
//            .unwrap_or(Ordering::Equal)
//    }
//}