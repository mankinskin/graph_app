pub mod parent;

use crate::trace::child::state::{ChildState};
use crate::path::{
    mutators::move_path::key::TokenPosition,
    structs::rooted::root::RootedPath,
};
use parent::ParentState;
use std::cmp::Ordering;

//pub trait SearchSpace {
//    fn expand(&mut self) -> Vec<TraceState<Self>>;
//}
// TODO:
// - param K: SearchSpace
// - connect input and internal path with Into<InternalState>
// AVAILABLE DATA FIELDS
// - PathBagMember<K> for each path for pos or id
// - PathBagProps<K> for whole bag for global pos
// AVAILABLE OPERATIONS
// - expand - (compare match, expand)
//

//
// TraceState<K> is either {ChildState<K>, ParentState<K>}
//

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BaseState<P: RootedPath> {
    pub prev_pos: TokenPosition,
    pub root_pos: TokenPosition,
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
