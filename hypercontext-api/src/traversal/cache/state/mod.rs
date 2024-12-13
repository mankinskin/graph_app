pub mod end;
pub mod child;
pub mod query;
pub mod parent;
pub mod start;
pub mod traversal;

use child::ChildState;
use end::EndState;
use itertools::Itertools;
use parent::ParentState;
use traversal::TraversalState;
use std::cmp::Ordering;

use crate::traversal::cache::{
    entry::new::NewEntry,
    key::prev::PrevKey,
};

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