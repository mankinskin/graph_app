use std::{
    fmt::Debug,
    ops::{
        Add,
        AddAssign,
    },
};

use crate::{
    direction::Right,
    graph::vertex::{
        VertexIndex,
        child::Child,
        has_vertex_index::HasVertexIndex,
        wide::Wide,
    },
    path::mutators::move_path::key::{
        MoveKey,
        TokenPosition,
    },
};

pub mod down;
pub mod up;
use down::{
    DownKey,
    DownPosition,
};
use up::{
    UpKey,
    UpPosition,
};

#[derive(Clone, Debug, Copy, Hash, Eq, PartialEq)]
pub enum DirectedPosition {
    BottomUp(UpPosition),
    TopDown(DownPosition),
}
pub trait HasTokenPosition {
    fn pos(&self) -> &TokenPosition;
}
impl HasTokenPosition for DirectedPosition {
    fn pos(&self) -> &TokenPosition {
        match self {
            Self::BottomUp(pos) => &pos.0,
            Self::TopDown(pos) => &pos.0,
        }
    }
}
impl DirectedPosition {
    pub fn flipped(self) -> Self {
        match self {
            Self::BottomUp(pos) => Self::TopDown(pos.flipped()),
            Self::TopDown(pos) => Self::BottomUp(pos.flipped()),
        }
    }
}

impl From<usize> for DirectedPosition {
    fn from(value: usize) -> Self {
        Self::BottomUp(value.into())
    }
}

impl Add<usize> for DirectedPosition {
    type Output = Self;
    fn add(
        self,
        rhs: usize,
    ) -> Self::Output {
        match self {
            Self::BottomUp(p) => Self::BottomUp(p + rhs),
            Self::TopDown(p) => Self::TopDown(p + rhs),
        }
    }
}

impl AddAssign<usize> for DirectedPosition {
    fn add_assign(
        &mut self,
        rhs: usize,
    ) {
        match self {
            Self::BottomUp(p) => *p += rhs,
            Self::TopDown(p) => *p += rhs,
        }
    }
}

impl MoveKey<Right> for DirectedPosition {
    type Delta = usize;
    fn move_key(
        &mut self,
        delta: Self::Delta,
    ) {
        match self {
            DirectedPosition::BottomUp(UpPosition(p)) =>
                <TokenPosition as MoveKey<Right>>::move_key(p, delta),
            DirectedPosition::TopDown(DownPosition(p)) =>
                <TokenPosition as MoveKey<Right>>::move_key(p, delta),
        }
    }
}

#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub struct DirectedKey {
    pub index: Child,
    pub pos: DirectedPosition,
}
impl Wide for DirectedKey {
    fn width(&self) -> usize {
        self.index.width()
    }
}
impl HasVertexIndex for DirectedKey {
    fn vertex_index(&self) -> VertexIndex {
        self.index.vertex_index()
    }
}
impl HasTokenPosition for DirectedKey {
    fn pos(&self) -> &TokenPosition {
        self.pos.pos()
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

impl DirectedKey {
    pub fn new(
        index: Child,
        pos: impl Into<DirectedPosition>,
    ) -> Self {
        Self {
            index,
            pos: pos.into(),
        }
    }
    pub fn up(
        index: Child,
        pos: impl Into<UpPosition>,
    ) -> Self {
        Self {
            index,
            pos: DirectedPosition::BottomUp(pos.into()),
        }
    }
    pub fn down(
        index: Child,
        pos: impl Into<DownPosition>,
    ) -> Self {
        Self {
            index,
            pos: DirectedPosition::TopDown(pos.into()),
        }
    }
    pub fn flipped(self) -> Self {
        Self {
            index: self.index,
            pos: self.pos.flipped(),
        }
    }
}

impl From<Child> for DirectedKey {
    fn from(index: Child) -> Self {
        Self {
            pos: DirectedPosition::BottomUp(index.width().into()),
            index,
        }
    }
}

impl From<UpKey> for DirectedKey {
    fn from(key: UpKey) -> Self {
        Self {
            index: key.index,
            pos: DirectedPosition::BottomUp(key.pos),
        }
    }
}

impl From<DownKey> for DirectedKey {
    fn from(key: DownKey) -> Self {
        Self {
            index: key.index,
            pos: DirectedPosition::TopDown(key.pos),
        }
    }
}
