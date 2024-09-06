use crate::traversal::path::mutators::move_path::key::TokenLocation;
use itertools::Itertools;
use std::cmp::Ordering;

pub mod end;

use end::*;

pub mod child;
pub mod query;

use child::*;

pub mod parent;

use parent::*;

pub mod start;

use crate::traversal::{
    cache::{
        entry::new::NewEntry,
        key::{
            prev::PrevKey,
            target::TargetKey,
        },
    },
    context::TraversalContext,
    iterator::TraversalIterator,
    path::accessors::{
        child::root::GraphRootChild,
        role::End,
    },
    result_kind::RoleChildPath,
};
use crate::graph::vertex::location::child::ChildLocation;

#[derive(Clone, Debug)]
pub struct StateNext<T> {
    pub prev: PrevKey,
    pub new: Vec<NewEntry>,
    pub inner: T,
}

#[derive(Clone, Debug)]
pub enum NextStates {
    Parents(StateNext<Vec<ParentState>>),
    Prefixes(StateNext<Vec<ChildState>>),
    End(StateNext<EndState>),
    Child(StateNext<ChildState>),
    Empty,
}

impl NextStates {
    pub fn into_states(self) -> Vec<TraversalState> {
        match self {
            Self::Parents(state) => state
                .inner
                .iter()
                .map(|s| TraversalState {
                    prev: state.prev,
                    new: state.new.clone(),
                    kind: InnerKind::Parent(s.clone()),
                })
                .collect_vec(),
            Self::Prefixes(state) => state
                .inner
                .iter()
                .map(|s| TraversalState {
                    prev: state.prev,
                    new: state.new.clone(),
                    kind: InnerKind::Child(s.clone()),
                })
                .collect_vec(),
            Self::Child(state) => vec![TraversalState {
                prev: state.prev,
                new: state.new,
                kind: InnerKind::Child(state.inner),
            }],
            Self::End(_) => vec![],
            Self::Empty => vec![],
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Copy, Hash)]
pub enum StateDirection {
    BottomUp,
    TopDown,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WaitingState {
    pub prev: PrevKey,
    pub state: ParentState,
}

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
    pub fn root_pos(&self) -> TokenLocation {
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
    pub fn next_states<'a, 'b: 'a, I: TraversalIterator<'b>>(
        mut self,
        ctx: &mut TraversalContext<'a, 'b, I>,
    ) -> Option<NextStates> {
        let key = self.target_key();
        let exists = ctx.cache.exists(&key);

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
                    state.next_states(ctx, self.new)
                } else {
                    // add other edges leading to this parent
                    for entry in self.new {
                        ctx.cache.add_state(ctx.trav(), entry, true);
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
