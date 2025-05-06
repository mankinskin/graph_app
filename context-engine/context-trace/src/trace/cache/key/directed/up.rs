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
    down::DownPosition,
};

#[derive(Clone, Debug, Copy, Hash, Eq, PartialEq, From)]
pub struct UpPosition(pub TokenPosition);

impl UpPosition {
    pub fn flipped(self) -> DownPosition {
        DownPosition(self.0)
    }
}

impl From<usize> for UpPosition {
    fn from(value: usize) -> Self {
        Self(value.into())
    }
}

impl AddAssign<usize> for UpPosition {
    fn add_assign(
        &mut self,
        rhs: usize,
    ) {
        self.0 += rhs;
    }
}

impl Add<usize> for UpPosition {
    type Output = Self;
    fn add(
        self,
        rhs: usize,
    ) -> Self::Output {
        Self(self.0 + rhs)
    }
}

#[derive(Clone, Debug, Hash, Eq, PartialEq, Copy, new)]
pub struct UpKey {
    pub index: Child,
    pub pos: UpPosition,
}

impl HasTokenPosition for UpKey {
    fn pos(&self) -> &TokenPosition {
        &self.pos.0
    }
}
