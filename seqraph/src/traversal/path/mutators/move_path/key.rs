use derive_more::{
    Add,
    Deref,
    DerefMut,
    Sub,
};

use crate::{
    direction::{
        Direction,
        Left,
        Right,
    },
    traversal::{
        cache::{
            key::{
                DirectedKey,
                DirectedPosition,
                DownPosition,
                UpPosition,
            },
            state::query::QueryState,
        },
        context::QueryStateContext,
    },
};

#[derive(Clone, Debug, Copy, Hash, Eq, PartialEq, Add, Sub, Deref, DerefMut)]
pub struct TokenLocation(pub usize);
impl Default for TokenLocation {
    fn default() -> Self {
        Self(0)
    }
}
impl From<usize> for TokenLocation {
    fn from(pos: usize) -> Self {
        Self(pos)
    }
}
impl std::ops::Add<usize> for TokenLocation {
    type Output = Self;
    fn add(
        mut self,
        delta: usize,
    ) -> Self {
        self.0 += delta;
        self
    }
}
impl std::ops::Sub<usize> for TokenLocation {
    type Output = Self;
    fn sub(
        mut self,
        delta: usize,
    ) -> Self {
        self.0 -= delta;
        self
    }
}
impl std::ops::AddAssign<usize> for TokenLocation {
    fn add_assign(
        &mut self,
        delta: usize,
    ) {
        self.0 += delta;
    }
}
impl std::ops::SubAssign<usize> for TokenLocation {
    fn sub_assign(
        &mut self,
        delta: usize,
    ) {
        self.0 -= delta;
    }
}

pub trait MoveKey<D: Direction> {
    type Delta = usize;
    fn move_key(
        &mut self,
        delta: Self::Delta,
    );
}
pub trait AdvanceKey: MoveKey<Right> {
    fn advance_key(
        &mut self,
        delta: Self::Delta,
    ) {
        self.move_key(delta)
    }
}
impl<T: MoveKey<Right>> AdvanceKey for T {}
pub trait RetractKey: MoveKey<Left> {
    fn retract_key(
        &mut self,
        delta: Self::Delta,
    ) {
        self.move_key(delta)
    }
}
impl<T: MoveKey<Left>> RetractKey for T {}

impl MoveKey<Right> for DirectedPosition {
    type Delta = usize;
    fn move_key(
        &mut self,
        delta: Self::Delta,
    ) {
        match self {
            DirectedPosition::BottomUp(UpPosition(p)) => {
                <TokenLocation as MoveKey<Right>>::move_key(p, delta)
            }
            DirectedPosition::TopDown(DownPosition(p)) => {
                <TokenLocation as MoveKey<Right>>::move_key(p, delta)
            }
        }
    }
}
impl MoveKey<Right> for DirectedKey {
    type Delta = usize;
    fn move_key(
        &mut self,
        delta: Self::Delta,
    ) {
        self.pos.move_key(delta)
    }
}
impl MoveKey<Right> for TokenLocation {
    type Delta = usize;
    fn move_key(
        &mut self,
        delta: Self::Delta,
    ) {
        *self += delta;
    }
}
impl MoveKey<Right> for QueryStateContext<'_> {
    type Delta = usize;
    fn move_key(
        &mut self,
        delta: Self::Delta,
    ) {
        self.state.advance_key(delta)
    }
}
impl MoveKey<Right> for QueryState {
    type Delta = usize;
    fn move_key(
        &mut self,
        delta: Self::Delta,
    ) {
        self.pos.advance_key(delta)
    }
}

impl MoveKey<Left> for TokenLocation {
    type Delta = usize;
    fn move_key(
        &mut self,
        delta: Self::Delta,
    ) {
        *self -= delta;
    }
}
impl MoveKey<Left> for QueryStateContext<'_> {
    type Delta = usize;
    fn move_key(
        &mut self,
        delta: Self::Delta,
    ) {
        self.state.retract_key(delta)
    }
}
impl MoveKey<Left> for QueryState {
    type Delta = usize;
    fn move_key(
        &mut self,
        delta: Self::Delta,
    ) {
        self.pos.retract_key(delta)
    }
}
