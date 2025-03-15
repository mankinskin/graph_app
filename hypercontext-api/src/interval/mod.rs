use std::{
    fmt::Debug,
    num::NonZeroUsize,
};

use derive_more::derive::{
    Deref,
    DerefMut,
};
use derive_new::new;

use crate::{
    graph::vertex::{
        child::Child,
        has_vertex_index::HasVertexIndex,
        wide::Wide,
    },
    traversal::{
        cache::{
            label_key::vkey::VertexCacheKey,
            TraversalCache,
        },
        fold::state::FoldState,
        split::{
            cache::{
                position::SplitPositionCache,
                vertex::SplitVertexCache,
                PosKey,
            },
            context::{
                SplitCacheContext,
                SplitTraceStateContext,
            },
            node::RootMode,
            SplitStates,
        },
        trace::context::TraceContext,
        traversable::TraversableMut,
    },
    HashMap,
};

pub mod partition;
pub(crate) mod side;

#[derive(Debug, Deref, DerefMut, new)]
pub struct SplitCache {
    pub root_mode: RootMode,

    #[deref]
    #[deref_mut]
    entries: HashMap<VertexCacheKey, SplitVertexCache>,
}
#[derive(Debug)]
pub struct IntervalGraph {
    pub states: SplitStates,
    pub cache: SplitCache,
    pub root: Child,
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
        let InitInterval {
            root,
            cache,
            end_bound,
        } = init;
        let ctx = TraceContext { trav, cache };
        let iter = SplitTraceStateContext::new(ctx, root, end_bound);
        Self::from(SplitCacheContext::init(iter))
    }
}
impl IntervalGraph {
    pub fn get(
        &self,
        key: &PosKey,
    ) -> Option<&SplitPositionCache> {
        self.cache
            .get(&key.index.vertex_index())
            .and_then(|ve| ve.positions.get(&key.pos))
    }
    pub fn get_mut(
        &mut self,
        key: &PosKey,
    ) -> Option<&mut SplitPositionCache> {
        self.cache
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
