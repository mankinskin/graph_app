use crate::trace::has_graph::HasGraph;
use std::{
    collections::VecDeque,
    fmt::Debug,
};

use super::state::ChildState;

pub type ChildQueue<S = ChildState> = VecDeque<S>;

impl From<ChildState> for ChildQueue {
    fn from(state: ChildState) -> Self {
        FromIterator::from_iter([state])
    }
}
//pub type ChildQueue = VecDeque<ChildModeCtx>;
pub trait QueuedState {}
impl<T> QueuedState for T {}

#[derive(Debug)]
pub struct ChildIterator<G: HasGraph, S: QueuedState = ChildState> {
    pub queue: ChildQueue<S>,
    pub trav: G,
}
impl<G: HasGraph, S: QueuedState> ChildIterator<G, S> {
    pub fn new(
        trav: G,
        queue: impl Into<ChildQueue<S>>,
    ) -> Self {
        Self {
            queue: queue.into(),
            trav,
        }
    }
}

impl<G: HasGraph, S: QueuedState> Iterator for ChildIterator<G, S> {
    type Item = S;
    fn next(&mut self) -> Option<Self::Item> {
        self.queue.pop_front()
    }
}
