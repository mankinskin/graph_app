use std::ops::ControlFlow;

use crate::path::mutators::path::{MovePath};

use crate::{
    direction::Right,
    path::accessors::role::End,
    trace::has_graph::HasGraph,
};

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
