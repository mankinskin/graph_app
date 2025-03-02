use std::{
    fmt::Debug,
    num::NonZeroUsize,
};

use partition::split::{
    PatternSplitPositions,
    VertexSplits,
};

use crate::traversal::cache::entry::position::SubSplitLocation;

pub mod cache;
pub mod partition;
pub(crate) mod side;
pub mod split;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct PatternSplitPos {
    pub inner_offset: Option<NonZeroUsize>,
    pub sub_index: usize,
}
impl From<(usize, Option<NonZeroUsize>)> for PatternSplitPos {
    fn from((sub_index, inner_offset): (usize, Option<NonZeroUsize>)) -> Self {
        Self {
            sub_index,
            inner_offset,
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
