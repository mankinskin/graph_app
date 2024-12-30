use std::cmp::Ordering;

use super::{InnerKind, NextStates, StateDirection, WaitingState};
use crate::{
    graph::vertex::location::child::ChildLocation, path::{
        accessors::{
            child::root::GraphRootChild,
            role::End,
        }, mutators::move_path::key::TokenPosition
    }, traversal::{
        cache::{
            entry::new::NewEntry,
            key::{
                prev::PrevKey,
                target::TargetKey,
            },
        }, fold::{TraversalContext, TraversalKind}, result::kind::RoleChildPath
    }
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TraversalState {
    pub prev: PrevKey,
    pub new: Vec<NewEntry>,
    pub kind: InnerKind,
}

impl From<WaitingState> for TraversalState {
    fn from(state: WaitingState) -> Self {
        Self {
            prev: state.prev,
            new: vec![],
            kind: InnerKind::Parent(state.state),
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

impl PartialOrd for TraversalState {
    fn partial_cmp(
        &self,
        other: &Self,
    ) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl TraversalState {
    pub fn entry_location(&self) -> Option<ChildLocation> {
        match &self.kind {
            InnerKind::Parent(state) => Some(state.path.root_child_location()),
            InnerKind::Child(state) => state.paths.path.role_leaf_child_location::<End>(),
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
    /// Retrieves next unvisited states and adds edges to cache
    pub fn next_states<'a, K: TraversalKind>(
        mut self,
        ctx: &mut TraversalContext<'a, K>,
    ) -> Option<NextStates> {
        let key = self.target_key();
        let exists = ctx.states.cache.exists(&key);

        //let prev = tstate.prev_key();
        //if !exists {
        //    cache.add_state((&tstate).into());
        //}
        if !exists && matches!(self.kind, InnerKind::Parent(_)) {
            self.new.push((&self).into());
        }
        let next_states = match self.kind {
            InnerKind::Parent(state) => {
                //debug!("Parent({}, {})", key.index.index(), key.index.width());
                if !exists {
                    state.next_states::<K>(ctx.trav, self.new)
                } else {
                    // add other edges leading to this parent
                    for entry in self.new {
                        ctx.states.cache.add_state(ctx.trav, entry, true);
                    }
                    NextStates::Empty
                }
            }
            InnerKind::Child(state) => {
                if !exists {
                    state.next_states(ctx, self.new)
                } else {
                    // add bottom up path
                    //state.trace(ctx.trav(), ctx.cache);
                    NextStates::Empty
                }
            }
        };
        Some(next_states)
    }
}
