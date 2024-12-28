use std::{
    collections::BTreeMap,
    iter::FromIterator,
    num::NonZeroUsize,
};

use crate::split::cache::position::SplitPositionCache;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct SplitVertexCache {
    pub positions: BTreeMap<NonZeroUsize, SplitPositionCache>,
}

impl SplitVertexCache {
    pub fn new(
        pos: NonZeroUsize,
        entry: SplitPositionCache,
    ) -> Self {
        Self {
            positions: BTreeMap::from_iter([(pos, entry)]),
        }
    }
    pub fn pos_mut(
        &mut self,
        pos: NonZeroUsize,
    ) -> &mut SplitPositionCache {
        self.positions.entry(pos).or_default()
    }
}
