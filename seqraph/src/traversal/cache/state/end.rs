
use crate::*;

// End types:
// - top down match-mismatch
// - top down match-query end
// - bottom up-no matching parents
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RangeEnd {
    pub path: SearchPath,
    pub target: DirectedKey,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PrefixEnd {
    pub path: RootedRolePath<End>,
    pub target: DirectedKey,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PostfixEnd {
    pub path: Primer,
    pub inner_width: usize,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EndState {
    pub reason: EndReason,
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
    pub fn waiting_root_key(&self) -> Option<DirectedKey> {
        match &self.kind {
            EndKind::Range(_) => Some(self.root_key()),
            EndKind::Postfix(_) => None,
            EndKind::Prefix(_) => None,
            EndKind::Complete(_) => None,
        }
    }
    pub fn end_path(&self) -> Option<RootedSplitPathRef<'_>> {
        match &self.kind {
            EndKind::Range(e) => Some(e.path.end_path()),
            EndKind::Postfix(_) => None,
            EndKind::Prefix(e) => Some((&e.path).into()),
            EndKind::Complete(_) => None,
        }
    }
    pub fn is_complete(&self) -> bool {
        matches!(self.kind, EndKind::Complete(_))
    }
    pub fn width(&self) -> usize {
        match &self.kind {
            EndKind::Range(p) => p.target.pos.pos().pos + p.target.index.width(),
            EndKind::Prefix(p) => p.target.pos.pos().pos + p.target.index.width(),
            EndKind::Postfix(p) => self.root_pos.pos + p.inner_width,
            EndKind::Complete(c) => c.width(),
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EndKind {
    Range(RangeEnd),
    Postfix(PostfixEnd),
    Prefix(PrefixEnd),
    Complete(Child),
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum EndReason {
    /// when the query has ended.
    QueryEnd,
    /// at a mismatch.
    Mismatch,
}
//impl From<MatchEnd<RootedRolePath<Start>>> for EndKind {
//    fn from(postfix: MatchEnd<RootedRolePath<Start>>) -> Self {
//        match postfix {
//            MatchEnd::Complete(c) => EndKind::Complete(c),
//            MatchEnd::Path(p) => EndKind::Postfix(
//                PostfixEnd {
//                    path: p.into(),
//                }
//            ),
//        }
//    }
//}
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