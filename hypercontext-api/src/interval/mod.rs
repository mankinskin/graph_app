use std::{
    fmt::Debug,
    num::NonZeroUsize,
};

use builder::IntervalGraphBuilder;
use partition::split::{
    PatternSplitPositions,
    VertexSplits,
};

use cache::{
    leaves::Leaves,
    position::SplitPositionCache,
    PosKey,
};
use derive_more::derive::{
    Deref,
    DerefMut,
};

use crate::{
    graph::vertex::{
        child::Child,
        has_vertex_index::HasVertexIndex,
        wide::Wide,
    },
    interval::cache::vertex::SplitVertexCache,
    traversal::{
        cache::{
            entry::{
                position::SubSplitLocation,
                RootMode,
            },
            label_key::vkey::VertexCacheKey,
            TraversalCache,
        },
        fold::state::FoldState,
        traversable::TraversableMut,
    },
    HashMap,
};

pub mod builder;
pub mod cache;
pub mod partition;
pub(crate) mod side;
pub mod split;

#[derive(Debug, Deref, DerefMut)]
pub struct SplitVertices {
    pub entries: HashMap<VertexCacheKey, SplitVertexCache>,
}
#[derive(Debug)]
pub struct IntervalGraph {
    pub vertices: SplitVertices,
    pub root_mode: RootMode,
    pub root: Child,
    pub leaves: Leaves,
}
#[derive(Debug)]
pub struct InitInterval {
    pub root: Child,
    pub cache: TraversalCache,
    pub end_bound: usize,
}
impl From<FoldState> for InitInterval {
    fn from(fold_state: FoldState) -> Self {
        Self {
            cache: fold_state.cache,
            root: fold_state.root,
            end_bound: fold_state.end_state.width(),
        }
    }
}
impl<'a, Trav: TraversableMut + 'a> From<(&'a mut Trav, InitInterval)> for IntervalGraph {
    fn from((trav, init): (&'a mut Trav, InitInterval)) -> Self {
        IntervalGraphBuilder::new(trav, init).build()
    }
}
impl IntervalGraph {
    pub fn get(
        &self,
        key: &PosKey,
    ) -> Option<&SplitPositionCache> {
        self.vertices
            .get(&key.index.vertex_index())
            .and_then(|ve| ve.positions.get(&key.pos))
    }
    pub fn get_mut(
        &mut self,
        key: &PosKey,
    ) -> Option<&mut SplitPositionCache> {
        self.vertices
            .get_mut(&key.index.vertex_index())
            .and_then(|ve| ve.positions.get_mut(&key.pos))
    }
    pub fn expect(
        &self,
        key: &PosKey,
    ) -> &SplitPositionCache {
        self.get(key).unwrap()
    }
    pub fn expect_mut(
        &mut self,
        key: &PosKey,
    ) -> &mut SplitPositionCache {
        self.get_mut(key).unwrap()
    }
}

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
