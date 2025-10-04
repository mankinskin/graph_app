use crate::{
    compare::{
        iterator::CompareIterator,
        parent::ParentCompareState,
        state::{
            ChildMatchState::*,
            CompareState,
        },
    },
    r#match::iterator::CompareParentBatch,
    traversal::{
        policy::DirectedTraversalPolicy,
        state::end::{
            EndKind,
            EndReason,
            EndState,
        },
        TraversalKind,
    },
};
use context_trace::*;
use std::collections::VecDeque;
pub type CompareQueue = VecDeque<CompareState>;

use std::{
    fmt::Debug,
    ops::ControlFlow::{
        self,
        Break,
        Continue,
    },
};
#[derive(Debug)]
pub struct RootCursor<G: HasGraph> {
    pub state: Box<CompareState>,
    pub trav: G,
}
impl<G: HasGraph> Iterator for RootCursor<G> {
    type Item = ControlFlow<EndReason>;

    fn next(&mut self) -> Option<Self::Item> {
        let prev_state = self.state.clone();
        match self.advanced() {
            Continue(_) => Some(
                // next position
                match CompareIterator::new(&self.trav, *self.state.clone())
                    .compare()
                {
                    Match(c) => {
                        *self.state = c;
                        Continue(())
                    },
                    Mismatch(_) => {
                        self.state = prev_state;
                        Break(EndReason::Mismatch)
                    },
                },
            ),
            // end of this root
            Break(None) => None,
            // end of query
            Break(Some(end)) => Some(Break(end)),
        }
    }
}
impl<G: HasGraph> RootCursor<G> {
    pub fn next_parents<K: TraversalKind>(
        self,
        trav: &K::Trav,
    ) -> Result<(ParentCompareState, CompareParentBatch), Box<EndState>> {
        let mut parent = self.state.parent_state();
        let prev_cursor = parent.cursor.clone();
        if parent.cursor.advance(trav).is_continue() {
            if let Some(batch) =
                K::Policy::next_batch(trav, &parent.parent_state)
            {
                let batch = CompareParentBatch {
                    batch,
                    cursor: parent.cursor.clone(),
                };
                Ok((parent, batch))
            } else {
                parent.cursor = prev_cursor;
                Err(Box::new(EndState::mismatch(trav, parent)))
            }
        } else {
            Err(Box::new(EndState::query_end(trav, parent)))
        }
    }
    fn advanced(&mut self) -> ControlFlow<Option<EndReason>> {
        if self.state.base.path.can_advance(&self.trav) {
            match self.query_advanced() {
                Continue(_) => {
                    let _ = self.path_advanced();
                    Continue(())
                },
                // end of query
                Break(_) => Break(Some(EndReason::QueryEnd)),
            }
        } else {
            // end of this root
            Break(None)
        }
    }
    fn query_advanced(&mut self) -> ControlFlow<()> {
        self.state.cursor.advance(&self.trav)
    }
    fn path_advanced(&mut self) -> ControlFlow<()> {
        self.state.base.path.advance(&self.trav)
    }
    pub fn find_end(mut self) -> Result<EndState, Self> {
        match self.find_map(|flow| match flow {
            Continue(()) => None,
            Break(reason) => Some(reason),
        }) {
            Some(reason) => {
                let CompareState {
                    child_state:
                        ChildState {
                            base: BaseState { path, root_pos, .. },
                            ..
                        },
                    cursor,
                    ..
                } = *self.state;
                let target_index = path.role_leaf_child::<End, _>(&self.trav);
                let pos = cursor.relative_pos;
                let target = DownKey::new(target_index, pos.into());
                Ok(EndState {
                    cursor,
                    reason,
                    kind: EndKind::from_range_path(
                        path, root_pos, target, &self.trav,
                    ),
                })
            },
            None => Err(self),
        }
    }
}
