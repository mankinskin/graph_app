use std::ops::ControlFlow;

use path::MovePath;

use crate::{
    direction::{
        Left,
        Right,
    },
    path::accessors::role::End,
    trace::has_graph::HasGraph,
};

pub mod key;
pub mod leaf;
pub mod path;
pub mod root;

pub trait Retract: MovePath<Left, End> {
    fn retract<G: HasGraph>(
        &mut self,
        trav: &G,
    ) -> ControlFlow<()> {
        self.move_path(trav)
    }
}

impl<T: MovePath<Left, End>> Retract for T {}

pub trait CanAdvance: Advance + Clone {
    fn can_advance<G: HasGraph>(
        &mut self,
        trav: &G,
    ) -> bool {
        self.clone().move_path(trav).is_continue()
    }
}
impl<T: Advance + Clone> CanAdvance for T {}
pub trait Advance: MovePath<Right, End> {
    fn advance<G: HasGraph>(
        &mut self,
        trav: &G,
    ) -> ControlFlow<()> {
        self.move_path(trav)
    }
}

impl<T: MovePath<Right, End>> Advance for T {}
