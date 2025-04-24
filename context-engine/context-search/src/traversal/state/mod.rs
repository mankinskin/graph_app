pub mod cursor;
pub(crate) mod traversal;

pub mod child;
pub mod end;
pub mod parent;
pub mod start;

use child::ChildState;
use cursor::PatternRangeCursor;
use parent::ParentState;
use std::cmp::Ordering;

use context_trace::path::{
    mutators::move_path::key::TokenPosition,
    structs::rooted::root::RootedPath,
};
#[derive(Clone, Debug)]
pub struct StateNext<T> {
    //pub prev: PrevKey,
    pub inner: T,
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

//impl From<InnerKind> for EditKind {
//    fn from(state: InnerKind) -> Self {
//        match state {
//            InnerKind::Parent(state) => Self::Parent(state.into()),
//            InnerKind::Child(state) => Self::Child(state.into()),
//        }
//    }
//}

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
