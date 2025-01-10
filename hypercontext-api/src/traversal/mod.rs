use container::StateContainer;
use iterator::policy::DirectedTraversalPolicy;
use traversable::Traversable;
use std::fmt::Debug;

pub mod cache;
pub mod fold;
pub mod iterator;
pub mod result;
pub mod traversable;
pub mod state;
pub mod trace;
pub mod container;
pub mod context;

pub trait TraversalKind: Debug {
    type Trav: Traversable;
    type Container: StateContainer;
    type Policy: DirectedTraversalPolicy<Trav = Self::Trav>;
}