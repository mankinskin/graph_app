use crate::traversal::{
    cache::{
        entry::new::NewEntry,
        key::{
            prev::ToPrev,
            target::TargetKey,
        },
        state::{
            end::{
                EndReason,
                EndState,
            },
            query::QueryState,
            NextStates,
            StateNext,
        },
    },
    context::TraversalContext,
    iterator::TraversalIterator,
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
    },
    policy::DirectedTraversalPolicy,
    result::kind::Primer,
};
use std::cmp::Ordering;

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
    pub fn next_states<'a, 'b: 'a, I: TraversalIterator<'b>>(
        self,
        ctx: &mut TraversalContext<'a, 'b, I>,
        new: Vec<NewEntry>,
    ) -> NextStates {
        let key = self.target_key();
        match self.into_advanced(ctx.trav()) {
            // first child state in this parent
            Ok(advanced) => {
                let delta = <_ as GraphRootChild<Start>>::root_post_ctx_width(
                    &advanced.paths.path,
                    ctx.trav(),
                );
                NextStates::Child(StateNext {
                    prev: key.flipped().to_prev(delta),
                    new,
                    inner: advanced,
                })
            }
            // no child state, bottom up path at end of parent
            Err(state) => state.next_parents(ctx, new),
        }
    }
    pub fn next_parents<'a, 'b: 'a, I: TraversalIterator<'b>>(
        self,
        ctx: &mut TraversalContext<'a, 'b, I>,
        new: Vec<NewEntry>,
    ) -> NextStates {
        // get next parents
        let key = self.target_key();
        let parents = I::Policy::next_parents(ctx.trav(), &self);
        let delta = self.path.root_post_ctx_width(ctx.trav());
        if parents.is_empty() {
            NextStates::End(StateNext {
                prev: key.to_prev(delta),
                new,
                inner: EndState {
                    reason: EndReason::Mismatch,
                    root_pos: self.root_pos,
                    kind: self.path.simplify(ctx.trav()),
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
