use super::{
    state::top_down::{
        child::{
            ChildCtx,
            ChildMatchState,
            ChildState,
        },
        end::{
            EndReason,
            EndState,
            RangeEnd,
        },
    },
    traversable::Traversable,
};
use crate::{
    graph::vertex::wide::Wide,
    path::{
        accessors::role::End,
        mutators::move_path::{
            key::AdvanceKey,
            Advance,
        },
        RoleChildPath,
    },
    traversal::{
        cache::key::directed::DirectedKey,
        BaseState,
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
pub struct RootCursor<Trav: Traversable> {
    pub state: ChildState,
    pub trav: Trav,
}
impl<Trav: Traversable> Iterator for RootCursor<Trav> {
    // iterator
    // None -> no end or next child found
    // Some(Continue) -> at next child position
    // Some(Break(EndReason)) -> at query end or mismatch
    type Item = ControlFlow<EndReason>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.query_advanced() {
            Continue(_) => match self.path_advanced() {
                Continue(_) => Some(
                    match ChildCtx::new(self.state.clone(), &self.trav).compare() {
                        ChildMatchState::Match(_) => Continue(()),
                        ChildMatchState::Mismatch(_) => Break(EndReason::Mismatch),
                    },
                ),
                // end of this root
                Break(_) => None,
            },
            // end of query
            Break(_) => Some(Break(EndReason::QueryEnd)),
        }
    }
}
impl<Trav: Traversable> RootCursor<Trav> {
    fn query_advanced(&mut self) -> ControlFlow<()> {
        self.state.cursor.advance(&self.trav)
    }
    fn path_advanced(&mut self) -> ControlFlow<()> {
        self.state.base.path.advance(&self.trav)
    }
    // Break: Found an end point in the root
    // Continue: Root matches fully
    pub fn find_end(mut self) -> Option<EndState> {
        (&mut self)
            .find_map(|flow| match flow {
                Continue(()) => None,
                Break(reason) => Some(reason),
            })
            .map(|reason| {
                let BaseState {
                    mut cursor,
                    path,
                    root_pos,
                    ..
                } = self.state.base;
                let target_index = path.role_leaf_child::<End, _>(&self.trav);
                let pos = cursor.relative_pos;
                cursor.advance_key(target_index.width());
                EndState {
                    root_pos,
                    cursor,
                    reason,
                    kind: RangeEnd {
                        path,
                        target: DirectedKey::down(target_index, pos),
                    }
                    .simplify_to_end(&self.trav),
                }
            })
        //if let Some(next) = root_cursor.next() {
        //    let primed = match next {
        //        MatchedNext::NextChild(next_child) => self
        //            .try_advance_query(next_child)
        //            .map(MatchedNext::NextChild),
        //        MatchedNext::MatchedParent(cs) => {
        //            self.try_advance_query(cs).map(MatchedNext::MatchedParent)
        //        }
        //    };
        //    // TODO: Root Candidate, cache root paths
        //    match primed {
        //        Ok(MatchedNext::NextChild(next_child)) => {
        //            self.children.extend([next_child]);
        //            None
        //        }
        //        Ok(MatchedNext::MatchedParent(next_child)) => {
        //            //ParentState {
        //            //    path: IndexStartPath::from(self.base.path),
        //            //    ..self.base
        //            //}
        //            //.next_parents::<K>(&ctx.trav)

        //            //self.parents.extend(
        //            //    next_child
        //            //        .root_parent
        //            //);
        //            None
        //        }
        //        Err(end) => Some(end),
        //    }
        //}

        //// TODO: cache candidate root paths
        //if self.cache.exists(&ps.target_key()) {
        //    // TODO: add edge to candidate cache
        //    Some((0, None))
        //} else {
        //    let end = match ps.parent_next_states::<K>(&self.trav) {
        //        ParentNext::Child(next_child) => {
        //            // TODO: process all parents in batch before processing children
        //            self.children.extend([next_child.inner]);
        //            None
        //        }
        //        ParentNext::BU(bu_next) => match bu_next {
        //            BUNext::Parents(p) => {
        //                self.parents.extend(p.inner);
        //                None
        //            }
        //            BUNext::End(end) => Some(end.inner),
        //        },
        //    };
        //    Some((0, end))
        //}
    }
}
