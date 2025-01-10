use crate::{
    path::{
        accessors::{
            child::root::GraphRootChild,
            role::Start,
            root::GraphRoot,
        },
        mutators::{
            adapters::into_advanced::IntoAdvanced,
            move_path::key::TokenPosition,
        },
    }, traversal::{
        cache::{
            entry::new::NewEntry,
            key::{
                prev::ToPrev,
                target::TargetKey,
            },
        }, iterator::policy::DirectedTraversalPolicy, result::kind::Primer, TraversalKind
    }
};
use std::cmp::Ordering;

use super::{
    end::{
        EndReason,
        EndState,
    },
    query::QueryState,
    NextStates,
    StateNext,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParentState {
    pub prev_pos: TokenPosition,
    pub root_pos: TokenPosition,
    pub path: Primer,
    pub query: QueryState,
}

impl Ord for ParentState {
    fn cmp(
        &self,
        other: &Self,
    ) -> Ordering {
        self.path.root_parent().cmp(&other.path.root_parent())
    }
}

impl PartialOrd for ParentState {
    fn partial_cmp(
        &self,
        other: &Self,
    ) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl ParentState {
    pub fn next_states<'a, K: TraversalKind>(
        self,
        trav: &K::Trav,
        new: Vec<NewEntry>,
    ) -> NextStates {
        let key = self.target_key();
        match self.into_advanced(trav) {
            // first child state in this parent
            Ok(advanced) => {
                let delta = <_ as GraphRootChild<Start>>::root_post_ctx_width(
                    &advanced.paths.path,
                    trav,
                );
                NextStates::Child(StateNext {
                    prev: key.flipped().to_prev(delta),
                    new,
                    inner: advanced,
                })
            }
            // no child state, bottom up path at end of parent
            Err(state) => state.next_parents::<K>(trav, new),
        }
    }
    pub fn next_parents<'a, K: TraversalKind>(
        self,
        trav: &K::Trav,
        new: Vec<NewEntry>,
    ) -> NextStates {
        // get next parents
        let key = self.target_key();
        let parents = K::Policy::next_parents(trav, &self);
        let delta = self.path.root_post_ctx_width(trav);
        if parents.is_empty() {
            NextStates::End(StateNext {
                prev: key.to_prev(delta),
                new,
                inner: EndState {
                    reason: EndReason::Mismatch,
                    root_pos: self.root_pos,
                    kind: self.path.simplify(trav),
                    query: self.query,
                },
            })
        } else {
            NextStates::Parents(StateNext {
                prev: key.to_prev(delta),
                new,
                inner: parents,
            })
        }
    }
}
