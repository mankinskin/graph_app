use std::{
    borrow::Borrow,
    num::NonZeroUsize,
    sync::RwLockWriteGuard,
};

use derive_more::{
    Deref,
    DerefMut,
};

use builder::*;
use ctx::*;
use position::*;

use crate::{
    graph::vertex::{
        child::Child,
        has_vertex_index::HasVertexIndex,
        location::SubLocation,
        pattern::{
            id::PatternId,
            Pattern,
        },
    }, partition::splits::offset::OffsetSplits, split::{
        cache::vertex::SplitVertexCache,
        PatternSplitPos,
    }, traversal::{
        cache::{
            entry::{
                position::SubSplitLocation,
                RootMode,
            },
            key::SplitKey,
            labelled_key::vkey::VertexCacheKey,
        },
        fold::state::FoldState,
        traversable::TraversableMut,
    }, HashMap
};

pub mod vertex;

pub mod builder;
pub mod ctx;
pub mod leaves;
pub mod position;
pub mod split;

#[derive(Debug, Clone)]
pub struct TraceState {
    pub index: Child,
    pub offset: NonZeroUsize,
    pub prev: SplitKey,
}

#[derive(Debug, Deref, DerefMut)]
pub struct SplitCache {
    pub entries: HashMap<VertexCacheKey, SplitVertexCache>,
    #[deref]
    #[deref_mut]
    pub context: CacheContext,
    pub root_mode: RootMode,
}

impl SplitCache {
    pub fn new<
        'a,
        Trav: TraversableMut<GuardMut<'a> = RwLockWriteGuard<'a, crate::graph::Hypergraph>> + 'a,
    >(
        trav: &'a mut Trav,
        fold_state: FoldState,
    ) -> Self {
        SplitCacheBuilder::new(trav, fold_state).build()
    }
    pub fn get(
        &self,
        key: &SplitKey,
    ) -> Option<&SplitPositionCache> {
        self.entries
            .get(&key.index.vertex_index())
            .and_then(|ve| ve.positions.get(&key.pos))
    }
    pub fn get_mut(
        &mut self,
        key: &SplitKey,
    ) -> Option<&mut SplitPositionCache> {
        self.entries
            .get_mut(&key.index.vertex_index())
            .and_then(|ve| ve.positions.get_mut(&key.pos))
    }
    pub fn expect(
        &self,
        key: &SplitKey,
    ) -> &SplitPositionCache {
        self.get(key).unwrap()
    }
    pub fn expect_mut(
        &mut self,
        key: &SplitKey,
    ) -> &mut SplitPositionCache {
        self.get_mut(key).unwrap()
    }
}

pub fn position_splits<'a>(
    patterns: impl IntoIterator<Item = (&'a PatternId, &'a Pattern)>,
    offset: NonZeroUsize,
) -> OffsetSplits {
    OffsetSplits {
        offset,
        splits: patterns
            .into_iter()
            .map(|(pid, pat)| {
                let (sub_index, inner_offset) =
                    IndexBack::token_offset_split(pat.borrow(), offset).unwrap();
                (
                    *pid,
                    PatternSplitPos {
                        sub_index,
                        inner_offset,
                    },
                )
            })
            .collect(),
    }
}

pub fn range_splits<'a>(
    patterns: impl Iterator<Item = (&'a PatternId, &'a Pattern)>,
    parent_range: (NonZeroUsize, NonZeroUsize),
) -> (OffsetSplits, OffsetSplits) {
    let (ls, rs) = patterns
        .map(|(pid, pat)| {
            let (li, lo) = IndexBack::token_offset_split(pat.borrow(), parent_range.0).unwrap();
            let (ri, ro) = IndexBack::token_offset_split(pat.borrow(), parent_range.1).unwrap();
            (
                (
                    *pid,
                    PatternSplitPos {
                        sub_index: li,
                        inner_offset: lo,
                    },
                ),
                (
                    *pid,
                    PatternSplitPos {
                        sub_index: ri,
                        inner_offset: ro,
                    },
                ),
            )
        })
        .unzip();
    (
        OffsetSplits {
            offset: parent_range.0,
            splits: ls,
        },
        OffsetSplits {
            offset: parent_range.1,
            splits: rs,
        },
    )
}

pub fn cleaned_position_splits<'a>(
    patterns: impl Iterator<Item = (&'a PatternId, &'a Pattern)>,
    parent_offset: NonZeroUsize,
) -> Result<Vec<SubSplitLocation>, SubLocation> {
    patterns
        .map(|(pid, pat)| {
            let (sub_index, inner_offset) =
                IndexBack::token_offset_split(pat.borrow(), parent_offset).unwrap();
            let location = SubLocation::new(*pid, sub_index);
            if inner_offset.is_some() || pat.len() > 2 {
                // can't be clean
                Ok(SubSplitLocation {
                    location,
                    inner_offset,
                })
            } else {
                // must be clean
                Err(location)
            }
        })
        .collect()
}
