use derive_more::{
    Add,
    Deref,
    DerefMut,
    Sub,
};

use crate::direction::{
    Direction,
    Left,
    Right,
};

#[derive(
    Clone, Debug, Copy, Hash, Eq, PartialEq, Add, Sub, Deref, DerefMut, Default,
)]
pub struct TokenPosition(pub usize);

impl Into<usize> for TokenPosition {
    fn into(self) -> usize {
        self.0
    }
}
impl From<usize> for TokenPosition {
    fn from(pos: usize) -> Self {
        Self(pos)
    }
}

impl std::ops::Add<usize> for TokenPosition {
    type Output = Self;
    fn add(
        mut self,
        delta: usize,
    ) -> Self {
        self.0 += delta;
        self
    }
}

impl std::ops::Sub<usize> for TokenPosition {
    type Output = Self;
    fn sub(
        mut self,
        delta: usize,
    ) -> Self {
        self.0 -= delta;
        self
    }
}

impl std::ops::AddAssign<usize> for TokenPosition {
    fn add_assign(
        &mut self,
        delta: usize,
    ) {
        self.0 += delta;
    }
}

impl std::ops::SubAssign<usize> for TokenPosition {
    fn sub_assign(
        &mut self,
        delta: usize,
    ) {
        self.0 -= delta;
    }
}

pub trait MoveKey<D: Direction> {
    fn move_key(
        &mut self,
        delta: usize,
    );
}

impl<D: Direction, T: MoveKey<D>> MoveKey<D> for &'_ mut T {
    fn move_key(
        &mut self,
        delta: usize,
    ) {
        (*self).move_key(delta)
    }
}

pub trait AdvanceKey: MoveKey<Right> {
    fn advance_key(
        &mut self,
        delta: usize,
    ) {
        self.move_key(delta)
    }
}

impl<T: MoveKey<Right>> AdvanceKey for T {}

pub trait RetractKey: MoveKey<Left> {
    fn retract_key(
        &mut self,
        delta: usize,
    ) {
        self.move_key(delta)
    }
}

impl<T: MoveKey<Left>> RetractKey for T {}

impl MoveKey<Right> for TokenPosition {
    fn move_key(
        &mut self,
        delta: usize,
    ) {
        *self += delta;
    }
}

impl MoveKey<Left> for TokenPosition {
    fn move_key(
        &mut self,
        delta: usize,
    ) {
        *self -= delta;
    }
}
