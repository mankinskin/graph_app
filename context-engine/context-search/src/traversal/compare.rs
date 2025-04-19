use super::state::{
    child::{
        batch::ChildIterator,
        ChildState,
    },
    end::{
        EndReason,
        EndState,
        RangeEnd,
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
            key::AdvanceKey,
            Advance,
        },
        RoleChildPath,
    },
    trace::{
        cache::key::directed::DirectedKey,
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
                    Match(_) => Continue(()),
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
        match self.query_advanced() {
            Continue(_) => match self.path_advanced() {
                Continue(_) => Continue(()),
                // end of this root
                Break(_) => Break(None),
            },
            // end of query
            Break(_) => Break(Some(EndReason::QueryEnd)),
        }
    }
    fn query_advanced(&mut self) -> ControlFlow<()> {
        self.state.cursor.advance(&self.trav)
    }
    fn path_advanced(&mut self) -> ControlFlow<()> {
        self.state.base.path.advance(&self.trav)
    }
    pub fn find_end(mut self) -> Option<EndState> {
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
                Some(EndState {
                    root_pos,
                    cursor,
                    reason,
                    kind: RangeEnd {
                        path,
                        target: DirectedKey::down(target_index, pos),
                    }
                    .simplify_to_end(&self.trav),
                })
            },
            _ => None,
        }
    }
}
