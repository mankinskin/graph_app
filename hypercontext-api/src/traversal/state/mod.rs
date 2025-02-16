pub mod cursor;
pub(crate) mod next_states;
pub(crate) mod traversal;

pub(crate) mod bottom_up;
pub mod top_down;

use bottom_up::parent::ParentState;
use std::cmp::Ordering;
use top_down::child::ChildState;

#[derive(Clone, Debug, PartialEq, Eq, Copy, Hash)]
pub enum StateDirection {
    BottomUp,
    TopDown,
}

//#[derive(Clone, Debug, PartialEq, Eq)]
//pub(crate) struct WaitingState {
//    pub prev: PrevKey,
//    pub state: ParentState,
//}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InnerKind {
    Parent(ParentState),
    Child(ChildState),
}

impl Ord for InnerKind {
    fn cmp(
        &self,
        other: &Self,
    ) -> Ordering {
        match (self, other) {
            (InnerKind::Child(a), InnerKind::Child(b)) => a.cmp(b),
            (InnerKind::Parent(a), InnerKind::Parent(b)) => a.cmp(b),
            (InnerKind::Child(_), _) => Ordering::Less,
            (_, InnerKind::Child(_)) => Ordering::Greater,
        }
    }
}

impl PartialOrd for InnerKind {
    fn partial_cmp(
        &self,
        other: &Self,
    ) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
