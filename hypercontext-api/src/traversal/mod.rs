use container::StateContainer;
use iterator::policy::DirectedTraversalPolicy;
use states::StatesContext;
use std::fmt::Debug;
use traversable::Traversable;

pub mod cache;
pub mod container;
pub mod fold;
pub mod iterator;
pub mod result;
pub mod state;
mod states;
mod trace;
pub mod traversable;

pub trait TraversalKind: Debug {
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
pub struct TraversalContext<'a, K: TraversalKind> {
    pub states: &'a mut StatesContext<K>,
    pub trav: &'a K::Trav,
}

impl<K: TraversalKind> Unpin for TraversalContext<'_, K> {}
