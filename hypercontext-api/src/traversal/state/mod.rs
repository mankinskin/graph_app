pub mod cursor;
pub(crate) mod next_states;
pub(crate) mod traversal;

pub mod bottom_up;
pub mod top_down;

use bottom_up::parent::ParentState;
use cursor::PatternRangeCursor;
use std::cmp::Ordering;
use top_down::child::ChildState;

use crate::path::{
    mutators::move_path::key::TokenPosition,
    structs::rooted::root::RootedPath,
};

#[derive(Clone, Debug, PartialEq, Eq, Copy, Hash)]
pub enum StateDirection {
    BottomUp,
    TopDown,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BaseState<P: RootedPath> {
    pub prev_pos: TokenPosition,
    pub root_pos: TokenPosition,
    pub cursor: PatternRangeCursor,
    pub path: P,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InnerKind {
    Parent(ParentState),
    Child(ChildState),
}
impl InnerKind {
    pub fn unwrap_parent(self) -> ParentState {
        if let Self::Parent(p) = self {
            p
        } else {
            panic!();
        }
    }
    pub fn unwrap_child(self) -> ChildState {
        if let Self::Child(c) = self {
            c
        } else {
            panic!();
        }
    }
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
