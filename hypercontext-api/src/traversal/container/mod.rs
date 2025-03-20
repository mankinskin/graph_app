use crate::traversal::state::traversal::TraversalState;
use extend::ExtendStates;
use std::fmt::Debug;

pub mod bft;
pub(crate) mod dft;
pub(crate) mod extend;
pub(crate) mod order;
pub(crate) mod pruning;

pub trait StateContainer:
    ExtendStates
    + Iterator<Item = (usize, TraversalState)>
    + Default
    + Debug
    + FromIterator<(usize, TraversalState)>
{
    fn clear(&mut self);
}
