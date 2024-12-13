use crate::{
    direction::{
        Direction,
        Left,
        Right,
        r#match::MatchDirection,
    },
    traversal::{
        context::QueryStateContext,
        traversable::{
            TravDir,
            Traversable,
        },
    },
    graph::vertex::wide::Wide,
};
use super::super::super::{
    accessors::{
        role::{
            End,
            PathRole,
        },
        root::RootPattern,
        child::pos::{
            RootChildPos,
            RootChildPosMut,
        },
    },
    mutators::move_path::key::{
        AdvanceKey,
        RetractKey,
    },
    structs::{
        query_range_path::QueryRangePath,
        rooted_path::{
            PathRoot,
            RootedRolePath,
            SearchPath,
        },
    },
};
use std::ops::ControlFlow;
use std::borrow::Borrow;

pub trait MoveRootPos<D: Direction, R: PathRole = End> {
    fn move_root_pos<Trav: Traversable>(
        &mut self,
        trav: &Trav,
    ) -> ControlFlow<()>;
}

impl<Root: PathRoot> MoveRootPos<Right, End> for RootedRolePath<End, Root> {
    fn move_root_pos<Trav: Traversable>(
        &mut self,
        _trav: &Trav,
    ) -> ControlFlow<()> {
        if let Some(next) = TravDir::<Trav>::index_next(RootChildPos::<End>::root_child_pos(self)) {
            *self.root_child_pos_mut() = next;
            ControlFlow::Continue(())
        } else {
            ControlFlow::Break(())
        }
    }
}

impl MoveRootPos<Right, End> for SearchPath {
    fn move_root_pos<Trav: Traversable>(
        &mut self,
        trav: &Trav,
    ) -> ControlFlow<()> {
        let graph = trav.graph();
        let pattern = self.root_pattern::<Trav>(&graph);
        if let Some(next) = TravDir::<Trav>::pattern_index_next(
            pattern.borrow(),
            RootChildPos::<End>::root_child_pos(self),
        ) {
            *self.root_child_pos_mut() = next;
            ControlFlow::Continue(())
        } else {
            ControlFlow::Break(())
        }
    }
}

impl MoveRootPos<Left, End> for SearchPath {
    fn move_root_pos<Trav: Traversable>(
        &mut self,
        trav: &Trav,
    ) -> ControlFlow<()> {
        let graph = trav.graph();
        let pattern = self.root_pattern::<Trav>(&graph);
        if let Some(prev) = TravDir::<Trav>::pattern_index_prev(
            pattern.borrow(),
            RootChildPos::<End>::root_child_pos(self),
        ) {
            *self.root_child_pos_mut() = prev;
            ControlFlow::Continue(())
        } else {
            ControlFlow::Break(())
        }
    }
}

impl MoveRootPos<Right, End> for QueryStateContext<'_> {
    fn move_root_pos<Trav: Traversable>(
        &mut self,
        _trav: &Trav,
    ) -> ControlFlow<()> {
        let pattern = &self.ctx.query_root;
        if let Some(next) =
            TravDir::<Trav>::pattern_index_next(pattern.borrow(), self.state.end.root_child_pos())
        {
            self.advance_key(pattern[self.state.end.root_child_pos()].width());
            *self.state.end.root_child_pos_mut() = next;
            ControlFlow::Continue(())
        } else {
            ControlFlow::Break(())
        }
    }
}

impl MoveRootPos<Left, End> for QueryStateContext<'_> {
    fn move_root_pos<Trav: Traversable>(
        &mut self,
        _trav: &Trav,
    ) -> ControlFlow<()> {
        let pattern = &self.ctx.query_root;
        if let Some(prev) =
            TravDir::<Trav>::pattern_index_prev(pattern.borrow(), self.state.end.root_child_pos())
        {
            self.retract_key(pattern[self.state.end.root_child_pos()].width());
            *self.state.end.root_child_pos_mut() = prev;
            ControlFlow::Continue(())
        } else {
            ControlFlow::Break(())
        }
    }
}

impl MoveRootPos<Right, End> for QueryRangePath {
    fn move_root_pos<Trav: Traversable>(
        &mut self,
        _trav: &Trav,
    ) -> ControlFlow<()> {
        if let Some(next) = TravDir::<Trav>::index_next(RootChildPos::<End>::root_child_pos(self)) {
            *self.root_child_pos_mut() = next;
            ControlFlow::Continue(())
        } else {
            ControlFlow::Break(())
        }
    }
}
