use crate::{
    traversal::{
        cache::{
            key::{
                prev::ToPrev,
                UpKey,
            },
            state::{
                end::{
                    EndKind,
                    EndReason,
                    EndState,
                },
                NextStates,
                query::QueryState,
                StateNext,
            },
        },
        context::TraversalStateContext,
        iterator::TraversalIterator,
        policy::DirectedTraversalPolicy,
    },
    path::mutators::{
        adapters::into_primer::IntoPrimer,
        move_path::{
            Advance,
            key::RetractKey,
        },
    },
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
    pub fn next_states<'a, 'b: 'a, I: TraversalIterator<'b>>(
        &mut self,
        ctx: &mut TraversalStateContext<'a, 'b, I>,
    ) -> NextStates
    where
        Self: 'a,
    {
        let mut query = self.query.to_ctx(ctx);
        let delta = self.index.width();
        if query.advance(ctx.trav()).is_continue() {
            // undo extra key advance
            query.retract_key(self.index.width());
            NextStates::Parents(StateNext {
                prev: self.key.to_prev(delta),
                new: vec![],
                inner: I::Policy::gen_parent_states(ctx.trav(), self.index, |trav, p| {
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
                    query: query.state.clone(),
                },
            })
        }
    }
}
