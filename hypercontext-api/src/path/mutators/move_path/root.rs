use crate::{
    direction::{
        r#match::MatchDirection,
        Direction,
        Left,
        Right,
    },
    graph::vertex::wide::Wide,
    path::{
        accessors::{
            child::pos::{
                RootChildPos,
                RootChildPosMut,
            },
            role::{
                End,
                PathRole,
            },
        },
        mutators::move_path::key::{
            AdvanceKey,
            RetractKey,
        },
        structs::rooted::{
            role_path::RootedRolePath,
            root::PathRoot,
        },
    },
    traversal::{
        //state::query::QueryState,
        state::cursor::RangeCursor,
        traversable::{
            TravDir,
            Traversable,
        },
    },
};
use std::{
    borrow::Borrow,
    ops::ControlFlow,
};

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

impl MoveRootPos<Right, End> for RangeCursor {
    fn move_root_pos<Trav: Traversable>(
        &mut self,
        _trav: &Trav,
    ) -> ControlFlow<()> {
        let pattern = &self.path.root;
        if let Some(next) =
            TravDir::<Trav>::pattern_index_next(pattern.borrow(), self.path.end.root_child_pos())
        {
            self.advance_key(pattern[self.path.end.root_child_pos()].width());
            *self.path.end.root_child_pos_mut() = next;
            ControlFlow::Continue(())
        } else {
            ControlFlow::Break(())
        }
    }
}

impl MoveRootPos<Left, End> for RangeCursor {
    fn move_root_pos<Trav: Traversable>(
        &mut self,
        _trav: &Trav,
    ) -> ControlFlow<()> {
        let pattern = &self.path.root;
        if let Some(prev) =
            TravDir::<Trav>::pattern_index_prev(pattern.borrow(), self.path.end.root_child_pos())
        {
            self.retract_key(pattern[self.path.end.root_child_pos()].width());
            *self.path.end.root_child_pos_mut() = prev;
            ControlFlow::Continue(())
        } else {
            ControlFlow::Break(())
        }
    }
}
