use std::cmp::Ordering;

use super::{
    child::ChildState,
    parent::ParentState,
    InnerKind,
    StateDirection,
};
use crate::{
    graph::vertex::location::child::ChildLocation,
    path::{
        accessors::{
            child::root::GraphRootChild,
            role::End,
        },
        mutators::move_path::key::TokenPosition,
        RoleChildPath,
    },
    traversal::cache::key::{
        directed::{
            up::UpKey,
            DirectedKey,
        },
        prev::PrevKey,
        props::{
            CursorPosition,
            RootKey,
            TargetKey,
        },
    },
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TraversalState {
    pub prev: PrevKey,
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
    pub fn prev_key(&self) -> PrevKey {
        self.prev
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

impl From<(PrevKey, ParentState)> for TraversalState {
    fn from((prev, ps): (PrevKey, ParentState)) -> Self {
        Self {
            prev,
            kind: InnerKind::Parent(ps),
        }
    }
}
impl From<(PrevKey, ChildState)> for TraversalState {
    fn from((prev, cs): (PrevKey, ChildState)) -> Self {
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

impl CursorPosition for TraversalState {
    fn cursor_pos(&self) -> &TokenPosition {
        match &self.kind {
            InnerKind::Parent(state) => state.cursor_pos(),
            InnerKind::Child(state) => state.cursor_pos(),
        }
    }
    fn cursor_pos_mut(&mut self) -> &mut TokenPosition {
        match &mut self.kind {
            InnerKind::Parent(state) => state.cursor_pos_mut(),
            InnerKind::Child(state) => state.cursor_pos_mut(),
        }
    }
}
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
