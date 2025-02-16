use std::{
    borrow::Borrow,
    num::NonZeroUsize,
};

use builder::*;
use ctx::*;
use derive_more::derive::{
    Deref,
    DerefMut,
};
use leaves::Leaves;
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
        wide::Wide,
    },
    interval::{
        cache::vertex::SplitVertexCache,
        partition::location::VertexSplits,
    },
    traversal::{
        cache::{
            entry::{
                position::SubSplitLocation,
                RootMode,
            },
            label_key::vkey::VertexCacheKey,
        },
        fold::state::FoldState,
        traversable::TraversableMut,
    },
    HashMap,
};

use super::side::{
    SplitBack,
    SplitSide,
};
use std::fmt::Debug;

pub mod vertex;

pub mod builder;
pub mod ctx;
pub mod leaves;
pub mod position;

#[derive(Clone, Debug, Copy, Hash, Eq, PartialEq)]
pub struct PosKey {
    pub index: Child,
    pub pos: NonZeroUsize,
}

impl PosKey {
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

impl From<Child> for PosKey {
    fn from(index: Child) -> Self {
        Self {
            index,
            pos: NonZeroUsize::new(index.width()).unwrap(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct TraceState {
    pub index: Child,
    pub offset: NonZeroUsize,
    pub prev: PosKey,
}

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
impl IntervalGraph {
    pub fn new<'a, Trav: TraversableMut + 'a>(
        trav: &'a mut Trav,
        fold_state: &mut FoldState,
    ) -> Self {
        IntervalGraphBuilder::new(trav, fold_state).build()
    }

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

pub fn position_splits<'a>(
    patterns: impl IntoIterator<Item = (&'a PatternId, &'a Pattern)>,
    pos: NonZeroUsize,
) -> VertexSplits {
    VertexSplits {
        pos,
        splits: patterns
            .into_iter()
            .map(|(pid, pat)| {
                let pos = SplitBack::token_pos_split(pat.borrow(), pos).unwrap();
                (*pid, pos)
            })
            .collect(),
    }
}

pub(crate) fn range_splits<'a>(
    patterns: impl Iterator<Item = (&'a PatternId, &'a Pattern)>,
    parent_range: (NonZeroUsize, NonZeroUsize),
) -> (VertexSplits, VertexSplits) {
    let (ls, rs) = patterns
        .map(|(pid, pat)| {
            let lpos = SplitBack::token_pos_split(pat.borrow(), parent_range.0).unwrap();
            let rpos = SplitBack::token_pos_split(pat.borrow(), parent_range.1).unwrap();
            ((*pid, lpos), (*pid, rpos))
        })
        .unzip();
    (
        VertexSplits {
            pos: parent_range.0,
            splits: ls,
        },
        VertexSplits {
            pos: parent_range.1,
            splits: rs,
        },
    )
}

pub(crate) fn cleaned_position_splits<'a>(
    patterns: impl Iterator<Item = (&'a PatternId, &'a Pattern)>,
    parent_offset: NonZeroUsize,
) -> Result<Vec<SubSplitLocation>, SubLocation> {
    patterns
        .map(|(pid, pat)| {
            let pos = SplitBack::token_pos_split(pat.borrow(), parent_offset).unwrap();
            let location = SubLocation::new(*pid, pos.sub_index);
            if pos.inner_offset.is_some() || pat.len() > 2 {
                // can't be clean
                Ok(SubSplitLocation {
                    location,
                    inner_offset: pos.inner_offset,
                })
            } else {
                // must be clean
                Err(location)
            }
        })
        .collect()
}
