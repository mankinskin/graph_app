use super::state::{
    child::{
        batch::ChildIterator,
        ChildState,
    },
    end::{
        EndKind,
        EndReason,
        EndState,
    },
};
use crate::traversal::{
    state::child::batch::ChildMatchState::{
        Match,
        Mismatch,
    },
    BaseState,
};
use context_trace::{
    graph::vertex::wide::Wide,
    path::{
        accessors::role::End,
        mutators::move_path::{
            advance::{
                Advance,
                CanAdvance,
            },
            key::AdvanceKey,
        },
        GetRoleChildPath,
    },
    trace::{
        cache::key::directed::down::DownKey,
        has_graph::HasGraph,
    },
};
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
    pub state: ChildState,
    pub trav: G,
}
impl<G: HasGraph> Iterator for RootCursor<G> {
    type Item = ControlFlow<EndReason>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.advanced() {
            Continue(_) => Some(
                // next position
                match ChildIterator::new(&self.trav, self.state.clone())
                    .compare()
                {
                    Match(c) => {
                        self.state = c;
                        Continue(())
                    },
                    Mismatch(_) => Break(EndReason::Mismatch),
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
    fn advanced(&mut self) -> ControlFlow<Option<EndReason>> {
        if self.state.base.path.can_advance(&self.trav) {
            match self.query_advanced() {
                Continue(_) => {
                    self.path_advanced();
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
        match (&mut self).find_map(|flow| match flow {
            Continue(()) => None,
            Break(reason) => Some(reason),
        }) {
            Some(reason) => {
                let BaseState {
                    mut cursor,
                    path,
                    root_pos,
                    ..
                } = self.state.base;
                let target_index = path.role_leaf_child::<End, _>(&self.trav);
                let pos = cursor.relative_pos;
                cursor.advance_key(target_index.width());
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
