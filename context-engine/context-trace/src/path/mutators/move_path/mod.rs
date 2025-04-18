use std::ops::ControlFlow;

use path::MovePath;

use crate::{
    direction::{
        Left,
        Right,
    },
    path::accessors::role::End,
    trace::traversable::Traversable,
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

pub trait CanAdvance: Advance + Clone {
    fn can_advance<Trav: Traversable>(
        &mut self,
        trav: &Trav,
    ) -> bool {
        self.clone().move_path(trav).is_continue()
    }
}
impl<T: Advance + Clone> CanAdvance for T {}
pub trait Advance: MovePath<Right, End> {
    fn advance<Trav: Traversable>(
        &mut self,
        trav: &Trav,
    ) -> ControlFlow<()> {
        self.move_path(trav)
    }
}

impl<T: MovePath<Right, End>> Advance for T {}
