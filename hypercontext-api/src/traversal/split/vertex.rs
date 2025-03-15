use std::{
    borrow::Borrow,
    num::NonZeroUsize,
};

use crate::{
    graph::vertex::pattern::id::PatternId,
    interval::PatternSplitPos,
    traversal::cache::entry::position::SubSplitLocation,
    HashMap,
};

use super::cache::position::SplitPositionCache;

#[derive(Debug, Clone, Copy)]
pub struct PosSplitContext<'a> {
    pub pos: &'a NonZeroUsize,
    pub split: &'a SplitPositionCache,
}

impl ToVertexSplits for PosSplitContext<'_> {
    fn to_vertex_splits(self) -> VertexSplits {
        VertexSplits {
            pos: *self.pos,
            splits: self.split.pattern_splits.clone(),
        }
    }
}

impl<'a, N: Borrow<(&'a NonZeroUsize, &'a SplitPositionCache)>> From<N> for PosSplitContext<'a> {
    fn from(item: N) -> Self {
        let (pos, split) = item.borrow();
        Self { pos, split }
    }
}
#[derive(Debug, Clone)]
pub struct VertexSplits {
    pub pos: NonZeroUsize,
    pub splits: PatternSplitPositions,
}

pub type PatternSplitPositions = HashMap<PatternId, PatternSplitPos>;

pub trait ToVertexSplits: Clone {
    fn to_vertex_splits(self) -> VertexSplits;
}

impl ToVertexSplits for VertexSplits {
    fn to_vertex_splits(self) -> VertexSplits {
        self
    }
}

impl ToVertexSplits for &VertexSplits {
    fn to_vertex_splits(self) -> VertexSplits {
        self.clone()
    }
}

impl<'a, N: Borrow<NonZeroUsize> + Clone, S: Borrow<SplitPositionCache> + Clone> ToVertexSplits
    for (N, S)
{
    fn to_vertex_splits(self) -> VertexSplits {
        VertexSplits::from(self)
    }
}
impl<'a, N: Borrow<NonZeroUsize>, S: Borrow<SplitPositionCache>> From<(N, S)> for VertexSplits {
    fn from(item: (N, S)) -> VertexSplits {
        VertexSplits {
            pos: *item.0.borrow(),
            splits: item.1.borrow().pattern_splits.clone(),
        }
    }
}

pub trait ToVertexSplitPos {
    fn to_vertex_split_pos(self) -> PatternSplitPositions;
}

impl ToVertexSplitPos for PatternSplitPositions {
    fn to_vertex_split_pos(self) -> PatternSplitPositions {
        self
    }
}

impl ToVertexSplitPos for Vec<SubSplitLocation> {
    fn to_vertex_split_pos(self) -> PatternSplitPositions {
        self.into_iter()
            .map(|loc| {
                (
                    loc.location.pattern_id,
                    PatternSplitPos {
                        inner_offset: loc.inner_offset,
                        sub_index: loc.location.sub_index,
                    },
                )
            })
            .collect()
    }
}

impl ToVertexSplitPos for VertexSplits {
    fn to_vertex_split_pos(self) -> PatternSplitPositions {
        self.splits
    }
}
