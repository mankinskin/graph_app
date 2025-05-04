use crate::traversal::{
    compare::RootCursor,
    state::{
        child::{
            batch::{
                ChildIterator,
                ChildMatchState,
                ChildQueue,
            },
            ChildState,
        },
        parent::{
            batch::ChildMatchState::{
                Match,
                Mismatch,
            },
            ParentState,
        },
    },
    HasGraph,
};
use context_trace::path::mutators::adapters::IntoAdvanced;
use derive_more::derive::{
    Deref,
    DerefMut,
};
use std::{
    collections::VecDeque,
    ops::ControlFlow::{
        self,
        Break,
        Continue,
    },
};
#[derive(Debug, Clone, Deref, DerefMut, Default)]
pub struct ParentBatch {
    pub parents: VecDeque<ParentState>,
}

#[derive(Debug)]
pub struct ParentBatchChildren<G: HasGraph> {
    pub child_iters: VecDeque<ChildQueue>,
    pub keep: Vec<ParentState>,
    pub trav: G,
}

impl<'a, G: HasGraph> ParentBatchChildren<G> {
    pub fn new(
        trav: G,
        batch: ParentBatch,
    ) -> Self {
        assert!(!batch.is_empty());

        let mut child_iters = VecDeque::default();
        let mut keep = Vec::default();
        for parent in batch.parents {
            match parent.into_advanced(&trav) {
                Ok(state) =>
                    child_iters.push_back(ChildQueue::from(state.child)),
                Err(parent) => {
                    keep.push(parent);
                },
            }
        }
        Self {
            child_iters,
            trav,
            keep,
        }
    }

    pub fn find_root_cursor(
        mut self
    ) -> ControlFlow<RootCursor<G>, Vec<ParentState>> {
        if let Some(child) = self.find_map(|root| root) {
            Break(RootCursor {
                trav: self.trav,
                state: child,
            })
        } else {
            Continue(self.keep)
        }
    }
}

impl<G: HasGraph> Iterator for ParentBatchChildren<G> {
    type Item = Option<ChildState>;

    fn next(&mut self) -> Option<Self::Item> {
        self.child_iters
            .pop_front()
            .map(|child_queue| {
                let mut child_iter =
                    ChildIterator::new(&self.trav, child_queue);
                child_iter.next().map(|res| match res {
                    Some(Match(cs)) => Some(cs),
                    Some(Mismatch(_)) => None,
                    None => {
                        self.child_iters.push_back(child_iter.children);
                        None
                    },
                })
            })
            .flatten()
    }
}
