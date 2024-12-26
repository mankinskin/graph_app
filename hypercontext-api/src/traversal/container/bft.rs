use crate::traversal::{
    fold::TraversalContext, state::traversal::TraversalState
};
use std::{
    cmp::Ordering,
    collections::BinaryHeap,
};

use super::{
    StateContainer,
    extend::ExtendStates,
};

pub type Bft<'a, K> = TraversalContext<'a, K>;

#[derive(Debug, Default)]
pub struct BftQueue {
    queue: BinaryHeap<QueueEntry>,
}

impl StateContainer for BftQueue {
    fn clear(&mut self) {
        self.queue.clear()
    }
}

impl Iterator for BftQueue {
    type Item = (usize, TraversalState);
    fn next(&mut self) -> Option<Self::Item> {
        self.queue.pop().map(|QueueEntry(d, s)| (d, s))
    }
}

impl ExtendStates for BftQueue {
    fn extend<
        It: DoubleEndedIterator + Iterator<Item = (usize, TraversalState)>,
        T: IntoIterator<Item = (usize, TraversalState), IntoIter = It>,
    >(
        &mut self,
        iter: T,
    ) {
        self.queue
            .extend(iter.into_iter().map(|(d, s)| QueueEntry(d, s)))
    }
}

#[derive(Debug, PartialEq, Eq)]
struct QueueEntry(usize, TraversalState);

impl Ord for QueueEntry {
    fn cmp(
        &self,
        other: &Self,
    ) -> Ordering {
        other.0.cmp(&self.0).then_with(|| self.1.cmp(&other.1))
    }
}

impl PartialOrd for QueueEntry {
    fn partial_cmp(
        &self,
        other: &Self,
    ) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
