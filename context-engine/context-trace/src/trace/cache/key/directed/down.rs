use std::ops::{
    Add,
    AddAssign,
};

use derive_more::derive::From;
use derive_new::new;

use crate::{
    graph::vertex::child::Child,
    path::mutators::move_path::key::TokenPosition,
};

use super::{
    HasTokenPosition,
    up::UpPosition,
};

#[derive(Clone, Debug, Copy, Hash, Eq, PartialEq, From)]
pub struct DownPosition(pub TokenPosition);

impl DownPosition {
    pub fn flipped(self) -> UpPosition {
        UpPosition(self.0)
    }
}
impl From<usize> for DownPosition {
    fn from(value: usize) -> Self {
        Self(value.into())
    }
}

impl AddAssign<usize> for DownPosition {
    fn add_assign(
        &mut self,
        rhs: usize,
    ) {
        self.0 += rhs;
    }
}

impl Add<usize> for DownPosition {
    type Output = Self;
    fn add(
        self,
        rhs: usize,
    ) -> Self::Output {
        Self(self.0 + rhs)
    }
}

#[derive(Clone, Debug, Hash, Eq, PartialEq, Copy, new)]
pub struct DownKey {
    pub index: Child,
    pub pos: DownPosition,
}

impl HasTokenPosition for DownKey {
    fn pos(&self) -> &TokenPosition {
        &self.pos.0
    }
}
