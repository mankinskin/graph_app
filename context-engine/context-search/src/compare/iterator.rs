use crate::compare::state::ChildMatchState::{
    self,
    Match,
    Mismatch,
};
use context_trace::*;

use std::fmt::Debug;

use crate::state::{CompareState};
use crate::compare::state::CompareNext::*;

#[derive(Debug)]
pub struct CompareIterator<G: HasGraph> {
    pub children: ChildIterator<G, CompareState>,
}

impl<G: HasGraph> CompareIterator<G> {
    pub fn new(
        trav: G,
        queue: impl Into<ChildQueue<CompareState>>,
    ) -> Self {
        Self {
            children: ChildIterator::new(trav, queue),
        }
    }
    pub fn find_match(self) -> Option<CompareState> {
        match self.compare() {
            Mismatch(_) => None,
            Match(state) => Some(state),
        }
    }
    pub fn compare(mut self) -> ChildMatchState {
        self.find_map(|flow| flow).unwrap()
    }
}
impl<G: HasGraph> Iterator for CompareIterator<G> {
    type Item = Option<ChildMatchState>;
    fn next(&mut self) -> Option<Self::Item> {
        self.children.next().map(|cs| {
            match cs.next_match(&self.children.trav) {
                Prefixes(next) => {
                    self.children.queue.extend(next);
                    None
                },
                MatchState(state) => Some(state),
            }
        })
    }
}
