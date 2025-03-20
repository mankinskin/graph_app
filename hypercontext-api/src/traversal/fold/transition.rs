use derive_more::derive::{
    Deref,
    DerefMut,
};

use crate::{
    graph::vertex::wide::Wide,
    traversal::{
        fold::FoldContext,
        state::next_states::{
            NextStates,
            StateNext,
        },
    },
};

use super::TraversalKind;
use crate::traversal::container::extend::ExtendStates;
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
            self.apply_transition(depth, next_states).then_some(())
        })
    }
}
impl<'a, K: TraversalKind> TransitionIter<'a, K> {
    pub fn apply_transition(
        &mut self,
        depth: usize,
        next_states: NextStates,
    ) -> bool {
        if let NextStates::End(StateNext { inner: end, .. }) = next_states {
            assert!(
                end.width() >= self.max_width,
                "Parents not evaluated in order"
            );
            let not_final = !end.is_final();
            if end.width() > self.max_width {
                self.max_width = end.width();
                self.fctx.end_state = Some(end);
            }
            not_final
        } else {
            self.fctx.tctx.states.extend(
                next_states
                    .into_states()
                    .into_iter()
                    .map(|nstate| (depth + 1, nstate)),
            );
            true
        }
    }
}
