use extend::ExtendStates;
use std::{
    cmp::Ordering,
    fmt::Debug,
};
use crate::{
    graph::vertex::wide::Wide, traversal::{
    cache::key::root::RootKey,
    state::traversal::TraversalState,
    traversable::Traversable,
}};
use crate::graph::vertex::location::child::ChildLocation;

pub mod extend;
pub mod dft;
pub mod bft;
pub mod pruning;
pub mod order;

pub trait StateContainer:
    ExtendStates + Iterator<Item = (usize, TraversalState)> + Default + Debug
{
    fn clear(&mut self);
}

