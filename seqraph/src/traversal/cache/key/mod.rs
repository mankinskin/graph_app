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
pub struct CacheKey {
    pub index: Child,
    pub pos: TokenLocation,
}
impl CacheKey {
    pub fn new(index: Child, pos: impl Into<TokenLocation>) -> Self {
        Self {
            index,
            pos: pos.into(),
        }
    }
}
impl From<Child> for CacheKey {
    fn from(index: Child) -> Self {
        Self {
            index,
            pos: index.width().into(),
        }
    }
}

pub trait GetCacheKey: RootKey + LeafKey {
    fn leaf_key(&self) -> CacheKey;
}

impl GetCacheKey for ChildState {
    fn leaf_key(&self) -> CacheKey {
        self.target
    }
}
