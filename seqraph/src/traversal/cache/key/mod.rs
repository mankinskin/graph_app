pub mod pos;
pub mod leaf;
pub mod root;
pub mod target;

use crate::*;
pub use pos::*; 
pub use leaf::*;
pub use root::*;
pub use target::*;

#[derive(Clone, Debug, Copy, Hash, Eq, PartialEq)]
pub enum DirectedPosition {
    BottomUp(TokenLocation),
    TopDown(TokenLocation),
}
impl DirectedPosition {
    pub fn pos(&self) -> &TokenLocation {
        match self {
            Self::BottomUp(pos) => pos,
            Self::TopDown(pos) => pos,
        }
    }
    pub fn flipped(self) -> Self {
        match self {
            Self::BottomUp(pos) => Self::TopDown(pos),
            Self::TopDown(pos) => Self::BottomUp(pos),
        }
    }
}
impl From<usize> for DirectedPosition {
    fn from(value: usize) -> Self {
        Self::BottomUp(value.into())
    }
}
#[derive(Clone, Debug, Copy, Hash, Eq, PartialEq)]
pub struct DirectedKey {
    pub index: Child,
    pub pos: DirectedPosition,
}
impl DirectedKey {
    pub fn new(index: Child, pos: impl Into<DirectedPosition>) -> Self {
        Self {
            index,
            pos: pos.into(),
        }
    }
    pub fn up(index: Child, pos: impl Into<TokenLocation>) -> Self {
        Self {
            index,
            pos: DirectedPosition::BottomUp(pos.into()),
        }
    }
    pub fn down(index: Child, pos: impl Into<TokenLocation>) -> Self {
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
    pub fn new(index: Child, pos: impl Into<NonZeroUsize>) -> Self {
        Self {
            index,
            pos: pos.into(),
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
