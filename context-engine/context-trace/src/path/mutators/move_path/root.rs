use crate::{
    direction::{
        Direction,
        Right,
        pattern::PatternDirection,
    },
    path::{
        accessors::{
            child::{
                RootChildPos,
                RootChildPosMut,
            },
            role::{
                End,
                PathRole,
            },
        },
        structs::rooted::{
            role_path::RootedRolePath,
            root::PathRoot,
        },
    },
    trace::traversable::{
        TravDir,
        Traversable,
    },
};
use std::ops::ControlFlow;

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
        if let Some(next) = TravDir::<Trav>::index_next(
            RootChildPos::<End>::root_child_pos(self),
        ) {
            *self.root_child_pos_mut() = next;
            ControlFlow::Continue(())
        } else {
            ControlFlow::Break(())
        }
    }
}
