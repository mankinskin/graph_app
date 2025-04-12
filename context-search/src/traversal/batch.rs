use std::{
    collections::VecDeque,
    ops::ControlFlow::{
        self,
        Break,
        Continue,
    },
};

use derive_more::derive::{
    Deref,
    DerefMut,
};

use super::{
    compare::RootCursor,
    state::{
        bottom_up::parent::ParentState,
        top_down::child::{
            ChildCtx,
            ChildState,
        },
    },
    traversable::Traversable,
};
use crate::path::mutators::adapters::IntoAdvanced;

#[derive(Debug, Clone, Deref, DerefMut, Default)]
pub struct ParentBatch {
    pub parents: VecDeque<ParentState>,
}
#[derive(Debug)]
pub struct ParentBatchChildren<Trav: Traversable> {
    pub batch: ParentBatch,
    pub keep: Vec<ParentState>,
    pub trav: Trav,
}

impl<Trav: Traversable> Iterator for ParentBatchChildren<Trav> {
    type Item = Option<(ParentState, ChildState)>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(parent) = self.batch.parents.pop_front() {
            // process parent
            match parent.into_advanced(&self.trav) {
                Ok(state) => Some(
                    ChildCtx::new(state.child, &self.trav)
                        .into_batch_iter()
                        .find_match()
                        .map(|child| (state.root_parent, child)),
                ),
                Err(parent) => {
                    self.keep.push(parent);
                    Some(None)
                }
            }
        } else {
            None
        }
    }
}

impl<Trav: Traversable> ParentBatchChildren<Trav> {
    pub fn new(
        trav: Trav,
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
    ) -> ControlFlow<(ParentState, RootCursor<Trav>), Vec<ParentState>> {
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
