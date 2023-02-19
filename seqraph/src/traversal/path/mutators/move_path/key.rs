use crate::*;

#[derive(Clone, Debug, Copy, Hash, Eq, PartialEq)]
pub struct TokenLocation {
    pub pos: usize,
}
impl Default for TokenLocation {
    fn default() -> Self {
        Self {
            pos: 0,
        }
    }
}
impl From<usize> for TokenLocation {
    fn from(pos: usize) -> Self {
        Self {
            pos,
        }
    }
}

pub trait MoveKey<D: Direction> {
    type Delta = usize;
    fn move_key(&mut self, delta: Self::Delta);
}
pub trait AdvanceKey: MoveKey<Right> {
    fn advance_key(&mut self, delta: Self::Delta) {
        self.move_key(delta)
    }
}
impl<T: MoveKey<Right>> AdvanceKey for T {
}
pub trait RetractKey: MoveKey<Left> {
    fn retract_key(&mut self, delta: Self::Delta) {
        self.move_key(delta)
    }
}
impl<T: MoveKey<Left>> RetractKey for T {
}

impl MoveKey<Right> for TokenLocation {
    type Delta = usize;
    fn move_key(&mut self, delta: Self::Delta) {
        self.pos += delta;
    }
}
impl MoveKey<Right> for CachedQuery<'_> {
    type Delta = usize;
    fn move_key(&mut self, delta: Self::Delta) {
        self.state.advance_key(delta)
    }
}
impl MoveKey<Right> for QueryState {
    type Delta = usize;
    fn move_key(&mut self, delta: Self::Delta) {
        self.pos.advance_key(delta)
    }
}

impl MoveKey<Left> for TokenLocation {
    type Delta = usize;
    fn move_key(&mut self, delta: Self::Delta) {
        self.pos -= delta;
    }
}
impl MoveKey<Left> for CachedQuery<'_> {
    type Delta = usize;
    fn move_key(&mut self, delta: Self::Delta) {
        self.state.retract_key(delta)
    }
}
impl MoveKey<Left> for QueryState {
    type Delta = usize;
    fn move_key(&mut self, delta: Self::Delta) {
        self.pos.retract_key(delta)
    }
}