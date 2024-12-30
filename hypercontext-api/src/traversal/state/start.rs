use crate::{
    path::mutators::{
        adapters::into_primer::IntoPrimer,
        move_path::{
            key::RetractKey, Advance
        },
    }, traversal::{
        cache::key::{
            prev::ToPrev,
            UpKey,
        }, fold::TraversalKind, iterator::policy::DirectedTraversalPolicy, state::{
            end::{
                EndKind,
                EndReason,
                EndState,
            }, query::QueryState, NextStates, StateNext
        }
    }
};
use crate::graph::vertex::{
    child::Child,
    wide::Wide,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StartState {
    pub index: Child,
    pub key: UpKey,
    pub query: QueryState,
}

impl StartState {
    pub fn next_states<'a, K: TraversalKind>(
        &mut self,
        trav: &K::Trav,
    ) -> NextStates
    where
        Self: 'a,
    {
        let delta = self.index.width();
        if self.query.advance(trav).is_continue() {
            // undo extra key advance
            self.query.retract_key(self.index.width());
            NextStates::Parents(StateNext {
                prev: self.key.to_prev(delta),
                new: vec![],
                inner: K::Policy::gen_parent_states(trav, self.index, |trav, p| {
                    (self.index, self.query.clone()).into_primer(trav, p)
                }),
            })
        } else {
            NextStates::End(StateNext {
                prev: self.key.to_prev(delta),
                new: vec![],
                inner: EndState {
                    reason: EndReason::QueryEnd,
                    root_pos: self.index.width().into(),
                    kind: EndKind::Complete(self.index),
                    query: self.query.clone(),
                },
            })
        }
    }
}
