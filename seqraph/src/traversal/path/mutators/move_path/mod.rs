pub mod path;
pub mod leaf;
pub mod root;
pub mod key;

pub use path::*;
pub use leaf::*;
pub use root::*;
pub use key::*;

use crate::shared::*;

pub trait Retract: MovePath<Left, End> {
    fn retract<
        Trav: Traversable,
    >(
        &mut self,
        trav: &Trav,
    ) -> ControlFlow<()> {
        self.move_path(trav)
    }
}
impl<T: MovePath<Left, End>> Retract for T
{
}

pub trait Advance: MovePath<Right, End> {
    fn advance<
        Trav: Traversable,
    >(
        &mut self,
        trav: &Trav,
    ) -> ControlFlow<()> {
        self.move_path(trav)
    }
}
impl<T: MovePath<Right, End>> Advance for T
{
}