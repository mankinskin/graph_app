use derive_more::derive::{
    Deref,
    DerefMut,
};

use crate::{
    graph::vertex::wide::Wide,
    traversal::{
        cache::key::props::RootKey,
        fold::FoldContext,
        state::next_states::{
            NextStates,
            StateNext,
        },
    },
};
use std::ops::ControlFlow;

use super::TraversalKind;
use crate::traversal::container::{
    extend::ExtendStates,
    pruning::PruneStates,
};
#[derive(Debug, Deref, DerefMut)]
pub struct TransitionIter<'a, K: TraversalKind> {
    #[deref_mut]
    #[deref]
    pub fctx: &'a mut FoldContext<K>,
}
impl<K: TraversalKind> Iterator for TransitionIter<'_, K> {
    type Item = ();

    fn next(&mut self) -> Option<Self::Item> {
        self.fctx.tctx.next().and_then(|(depth, next_states)| {
            self.apply_transition(depth, next_states).continue_value()
        })
    }
}
impl<'a, K: TraversalKind> TransitionIter<'a, K> {
    pub fn apply_transition(
        &mut self,
        depth: usize,
        next_states: NextStates,
    ) -> ControlFlow<()> {
        match next_states {
            NextStates::Empty => ControlFlow::Continue(()),
            NextStates::Child(_) | NextStates::Prefixes(_) | NextStates::Parents(_) => {
                self.fctx.tctx.states.extend(
                    next_states
                        .into_states()
                        .into_iter()
                        .map(|nstate| (depth + 1, nstate)),
                );
                ControlFlow::Continue(())
            }
            NextStates::End(StateNext { inner: end, .. }) => {
                if end.width() >= self.max_width {
                    self.max_width = end.width();
                    let is_final = end.is_final();
                    self.fctx.end_state = Some(end);
                    if is_final {
                        ControlFlow::Break(())
                    } else {
                        ControlFlow::Continue(())
                    }
                } else {
                    // larger root already found
                    // stop other paths with this root
                    self.fctx.tctx.states.prune_below(end.root_key());
                    ControlFlow::Continue(())
                }
            }
        }
    }
}
