use cache::{
    key::props::TargetKey,
    TraversalCache,
};
use container::StateContainer;
use fold::states::PrunedStates;
use iterator::policy::DirectedTraversalPolicy;
use state::{
    next_states::NextStates,
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
        if let Some((depth, ts)) = self.states.next() {
            let next = match ts.kind.clone() {
                InnerKind::Parent(ps) => {
                    if self.cache.exists(&ps.target_key()) {
                        self.cache.add_state(&self.trav, ts, true);
                        NextStates::Empty
                    } else {
                        ps.parent_next_states::<K>(&self.trav, ts.prev)
                    }
                }
                InnerKind::Child(cs) => cs.child_next_states(self),
            };
            Some((depth, next))
        } else {
            None
        }
    }
}

impl<K: TraversalKind> Unpin for TraversalContext<K> {}
