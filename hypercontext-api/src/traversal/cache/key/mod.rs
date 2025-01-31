pub mod leaf;
pub mod pos;
pub mod prev;
pub mod root;
pub mod target;

use derive_more::From;
use derive_new::new;
use std::{
    convert::TryInto,
    fmt::Debug,
    num::NonZeroUsize,
    ops::{
        Add,
        AddAssign,
    },
};

use crate::{
    graph::vertex::{
        child::Child,
        wide::Wide,
    },
    path::mutators::move_path::key::TokenPosition,
    traversal::state::child::ChildState,
};
use leaf::*;
use root::*;

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

#[derive(Clone, Debug, Copy, Hash, Eq, PartialEq, new)]
pub struct UpKey {
    pub index: Child,
    pub pos: UpPosition,
}

#[derive(Clone, Debug, Copy, Hash, Eq, PartialEq, new)]
pub struct DownKey {
    pub index: Child,
    pub pos: DownPosition,
}

#[derive(Clone, Debug, Copy, Hash, Eq, PartialEq)]
pub enum DirectedPosition {
    BottomUp(UpPosition),
    TopDown(DownPosition),
}

impl DirectedPosition {
    pub fn pos(&self) -> &TokenPosition {
        match self {
            Self::BottomUp(pos) => &pos.0,
            Self::TopDown(pos) => &pos.0,
        }
    }
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

#[derive(Clone, Debug, Copy, Hash, Eq, PartialEq)]
pub struct DirectedKey {
    pub index: Child,
    pub pos: DirectedPosition,
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
            index,
            pos: DirectedPosition::BottomUp(index.width().into()),
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

pub trait GetCacheKey: RootKey + LeafKey {
    fn leaf_key(&self) -> DirectedKey;
}

impl GetCacheKey for ChildState {
    fn leaf_key(&self) -> DirectedKey {
        self.target
    }
}

#[derive(Clone, Debug, Copy, Hash, Eq, PartialEq)]
pub struct SplitKey {
    pub index: Child,
    pub pos: NonZeroUsize,
}

impl SplitKey {
    pub fn new<P: TryInto<NonZeroUsize>>(
        index: Child,
        pos: P,
    ) -> Self
    where
        P::Error: Debug,
    {
        Self {
            index,
            pos: pos.try_into().unwrap(),
        }
    }
}

impl From<Child> for SplitKey {
    fn from(index: Child) -> Self {
        Self {
            index,
            pos: NonZeroUsize::new(index.width()).unwrap(),
        }
    }
}
