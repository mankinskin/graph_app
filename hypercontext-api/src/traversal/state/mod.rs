pub(crate) mod child;
pub mod cursor;
pub mod end;
pub(crate) mod parent;
pub(crate) mod start;
pub(crate) mod traversal;

use child::ChildState;
use end::{
    EndKind,
    EndReason,
    EndState,
};
use itertools::Itertools;
use parent::ParentState;
use std::{
    cmp::Ordering,
    ops::ControlFlow,
};
use traversal::TraversalState;

use crate::{
    graph::vertex::wide::Wide,
    traversal::{
        cache::{
            entry::new::NewEntry,
            key::prev::PrevKey,
        },
        trace::TraceContext,
    },
};

use super::{
    cache::key::root::RootKey,
    container::{
        extend::ExtendStates,
        pruning::PruneStates,
    },
    trace::TraceInit,
    TraversalContext,
    TraversalKind,
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

#[derive(Debug)]
pub struct ApplyStatesCtx<'a: 'b, 'b, K: TraversalKind> {
    pub tctx: &'a mut TraversalContext<'b, K>,
    pub depth: usize,
    pub max_width: &'a mut usize,
    pub end_state: &'a mut Option<EndState>,
}
impl<'a: 'b, 'b, K: TraversalKind> ApplyStatesCtx<'a, 'b, K> {
    pub fn apply_transition(
        self,
        next_states: NextStates,
    ) -> ControlFlow<()> {
        match next_states {
            NextStates::Child(_) | NextStates::Prefixes(_) | NextStates::Parents(_) => {
                self.tctx.states.extend(
                    next_states
                        .into_states()
                        .into_iter()
                        .map(|nstate| (self.depth + 1, nstate)),
                );
                ControlFlow::Continue(())
            }
            NextStates::Empty => ControlFlow::Continue(()),
            NextStates::End(StateNext { inner: end, .. }) => {
                //debug!("{:#?}", state);
                if end.width() >= *self.max_width {
                    end.trace(&mut TraceContext {
                        cache: &mut self.tctx.states.cache,
                        trav: self.tctx.trav,
                    });

                    // note: not really needed with completion
                    //if let Some(root_key) = end.waiting_root_key() {
                    //    // continue paths also arrived at this root
                    //    // this must happen before simplification
                    //    states.extend(
                    //        cache.continue_waiting(&root_key)
                    //    );
                    //}
                    if end.width() > *self.max_width {
                        *self.max_width = end.width();
                        //end_states.clear();
                    }
                    let is_final = end.reason == EndReason::QueryEnd
                        && matches!(end.kind, EndKind::Complete(_));
                    *self.end_state = Some(end);
                    if is_final {
                        ControlFlow::Break(())
                    } else {
                        ControlFlow::Continue(())
                    }
                } else {
                    // larger root already found
                    // stop other paths with this root
                    self.tctx.states.prune_below(end.root_key());
                    ControlFlow::Continue(())
                }
            }
        }
    }
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
