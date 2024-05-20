pub mod key;
pub mod leaf;
pub mod path;
pub mod root;

use crate::{
    direction::{
        Left,
        Right,
    },
    traversal::{
        path::accessors::role::End,
        traversable::Traversable,
    },
};
pub use key::*;
pub use leaf::*;
pub use path::*;
pub use root::*;
use std::ops::ControlFlow;

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
