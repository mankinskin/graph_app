use cache::{
    key::props::TargetKey,
    TraversalCache,
};
use container::StateContainer;
use fold::states::PrunedStates;
use iterator::policy::DirectedTraversalPolicy;
use state::{
    next_states::NextStates,
    traversal::TraversalState,
    InnerKind,
};
use std::fmt::Debug;
use traversable::Traversable;

pub mod cache;
pub mod container;
pub mod fold;
pub mod iterator;
pub mod result;
pub mod split;
pub mod state;
pub mod trace;
pub mod traversable;

pub trait TraversalKind: Debug + Default {
    type Trav: Traversable;
    type Container: StateContainer;
    type Policy: DirectedTraversalPolicy<Trav = Self::Trav>;
}

//  1. Input
//      - Pattern
//      - QueryState
//  2. Init
//      - Trav
//      - start index
//      - start states
//  3. Fold
//      - TraversalCache
//      - FoldStepState

/// context for generating next states
#[derive(Debug)]
pub struct TraversalContext<K: TraversalKind> {
    pub states: PrunedStates<K>,
    pub cache: TraversalCache,
    pub trav: K::Trav,
}

impl<K: TraversalKind> Iterator for TraversalContext<K> {
    type Item = (usize, NextStates);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some((depth, tstate)) = self.states.next() {
            self.traversal_next_states(tstate)
                .map(|states| (depth, states))
        } else {
            None
        }
    }
}
impl<K: TraversalKind> TraversalContext<K> {
    /// Retrieves next unvisited states and adds edges to cache
    pub fn traversal_next_states(
        &mut self,
        mut tstate: TraversalState,
    ) -> Option<NextStates> {
        let key = tstate.target_key();
        let exists = self.cache.exists(&key);

        //let prev = tstate.prev_key();
        //if !exists {
        //    cache.add_state((&tstate).into());
        //}
        if !exists && matches!(tstate.kind, InnerKind::Parent(_)) {
            tstate.new.push((&tstate).into());
        }
        let next_states = match tstate.kind {
            InnerKind::Parent(ps) => {
                //debug!("Parent({}, {})", key.index.index(), key.index.width());
                if !exists {
                    ps.parent_next_states::<K>(&self.trav, tstate.new)
                } else {
                    // add other edges leading to this parent
                    for entry in tstate.new {
                        self.cache.add_state(&self.trav, entry, true);
                    }
                    NextStates::Empty
                }
            }
            InnerKind::Child(cs) => {
                if !exists {
                    cs.child_next_states(self, tstate.new)
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

impl<K: TraversalKind> Unpin for TraversalContext<K> {}
