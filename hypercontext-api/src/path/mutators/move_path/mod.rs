use std::ops::ControlFlow;


use path::MovePath;

use crate::{
    direction::{
        Left,
        Right,
    },
    traversal::traversable::Traversable,
    path::accessors::role::End,
};

pub mod key;
pub mod leaf;
pub mod path;
pub mod root;

pub trait Retract: MovePath<Left, End> {
    fn retract<Trav: Traversable>(
        &mut self,
        trav: &Trav,
    ) -> ControlFlow<()> {
        self.move_path(trav)
    }
}

impl<T: MovePath<Left, End>> Retract for T {}

pub trait Advance: MovePath<Right, End> {
    fn advance<Trav: Traversable>(
        &mut self,
        trav: &Trav,
    ) -> ControlFlow<()> {
        self.move_path(trav)
    }
}

impl<T: MovePath<Right, End>> Advance for T {}
