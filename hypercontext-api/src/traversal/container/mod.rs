use crate::traversal::state::traversal::TraversalState;
use extend::ExtendStates;
use std::fmt::Debug;

pub mod bft;
pub mod dft;
pub mod extend;
pub mod order;
pub mod pruning;

pub trait StateContainer:
    ExtendStates + Iterator<Item = (usize, TraversalState)> + Default + Debug
{
    fn clear(&mut self);
}
