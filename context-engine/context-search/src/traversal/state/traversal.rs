use std::cmp::Ordering;

use context_trace::{
    graph::vertex::location::child::ChildLocation,
    path::{
        accessors::{
            child::root::GraphRootChild,
            role::End,
        },
        mutators::move_path::key::TokenPosition,
        RolePathUtils,
    },
    trace::{
        cache::key::{
            directed::{
                up::UpKey,
                DirectedKey,
            },
            props::{
                RootKey,
                TargetKey,
            },
        },
        state::InnerKind,
        StateDirection,
    },
};
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TraversalState {
    pub prev: DirectedKey,
    pub kind: InnerKind,
}
impl TraversalState {
    pub fn entry_location(&self) -> Option<ChildLocation> {
        match &self.kind {
            InnerKind::Parent(state) => Some(state.path.root_child_location()),
            InnerKind::Child(state) =>
                state.path.role_leaf_child_location::<End>(),
        }
    }
    pub fn prev_key(&self) -> DirectedKey {
        self.prev.clone()
    }
    pub fn root_pos(&self) -> TokenPosition {
        match &self.kind {
            InnerKind::Parent(state) => state.root_pos,
            InnerKind::Child(state) => state.root_pos,
        }
    }

    pub fn state_direction(&self) -> StateDirection {
        match &self.kind {
            InnerKind::Parent(_) => StateDirection::BottomUp,
            InnerKind::Child(_) => StateDirection::TopDown,
        }
    }
}

//impl From<TraversalState> for NewEntry {
//    fn from(state: TraversalState) -> Self {
//        Self {
//            prev: state.prev_key(),
//            //root_pos: state.root_pos(),
//            kind: state.kind.into(),
//        }
//    }
//}
use context_trace::trace::{
    child::state::ChildState,
    state::parent::ParentState,
};
impl From<(DirectedKey, ParentState)> for TraversalState {
    fn from((prev, ps): (DirectedKey, ParentState)) -> Self {
        Self {
            prev,
            kind: InnerKind::Parent(ps),
        }
    }
}
impl From<(DirectedKey, ChildState)> for TraversalState {
    fn from((prev, cs): (DirectedKey, ChildState)) -> Self {
        Self {
            prev,
            kind: InnerKind::Child(cs),
        }
    }
}
impl TargetKey for TraversalState {
    fn target_key(&self) -> DirectedKey {
        match &self.kind {
            InnerKind::Parent(state) => state.target_key(),
            InnerKind::Child(state) => state.target_key(),
        }
    }
}

//impl CursorPosition for TraversalState {
//    fn cursor_pos(&self) -> &TokenPosition {
//        match &self.kind {
//            InnerKind::Parent(state) => state.cursor_pos(),
//            InnerKind::Child(state) => state.cursor_pos(),
//        }
//    }
//    fn cursor_pos_mut(&mut self) -> &mut TokenPosition {
//        match &mut self.kind {
//            InnerKind::Parent(state) => state.cursor_pos_mut(),
//            InnerKind::Child(state) => state.cursor_pos_mut(),
//        }
//    }
//}
impl Ord for TraversalState {
    fn cmp(
        &self,
        other: &Self,
    ) -> Ordering {
        self.kind.cmp(&other.kind)
    }
}
impl RootKey for TraversalState {
    fn root_key(&self) -> UpKey {
        match &self.kind {
            InnerKind::Parent(state) => state.root_key(),
            InnerKind::Child(state) => state.root_key(),
        }
    }
}

impl PartialOrd for TraversalState {
    fn partial_cmp(
        &self,
        other: &Self,
    ) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
