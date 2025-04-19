use crate::traversal::{
    compare::RootCursor,
    state::{
        child::{
            batch::ChildIterator,
            ChildState,
        },
        parent::ParentState,
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
    pub batch: ParentBatch,
    pub keep: Vec<ParentState>,
    pub trav: G,
}

impl<'a, G: HasGraph> ParentBatchChildren<G> {
    pub fn new(
        trav: G,
        batch: ParentBatch,
    ) -> Self {
        assert!(!batch.is_empty());

        Self {
            batch,
            trav,
            keep: Default::default(),
        }
    }

    pub fn find_root_cursor(
        mut self
    ) -> ControlFlow<(ParentState, RootCursor<G>), Vec<ParentState>> {
        if let Some((root_parent, child)) = self.find_map(|root| root) {
            Break((
                root_parent,
                RootCursor {
                    trav: self.trav,
                    state: child,
                },
            ))
        } else {
            Continue(self.keep)
        }
    }
}

impl<G: HasGraph> Iterator for ParentBatchChildren<G> {
    type Item = Option<(ParentState, ChildState)>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(parent) = self.batch.parents.pop_front() {
            // process parent
            match parent.into_advanced(&self.trav) {
                Ok(state) => Some(
                    ChildIterator::new(&self.trav, state.child)
                        .find_match()
                        .map(|child| (state.root_parent, child)),
                ),
                Err(parent) => {
                    self.keep.push(parent);
                    Some(None)
                },
            }
        } else {
            None
        }
    }
}
