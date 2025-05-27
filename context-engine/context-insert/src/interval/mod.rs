use std::fmt::Debug;

use crate::split::{
    cache::{
        SplitCache,
        position::{
            PosKey,
            SplitPositionCache,
        },
    },
    context::SplitCacheContext,
    trace::states::{
        SplitStates,
        context::SplitTraceStatesContext,
    },
};
use context_search::traversal::result::IncompleteState;
use context_trace::{
    graph::vertex::{
        child::Child,
        has_vertex_index::HasVertexIndex,
        wide::Wide,
    },
    trace::{
        TraceContext,
        cache::TraceCache,
        has_graph::HasGraphMut,
    },
};

pub mod partition;

#[derive(Debug)]
pub struct IntervalGraph {
    pub states: SplitStates,
    pub cache: SplitCache,
    pub root: Child,
}
#[derive(Debug)]
pub struct InitInterval {
    pub root: Child,
    pub cache: TraceCache,
    pub end_bound: usize,
}
impl From<IncompleteState> for InitInterval {
    fn from(state: IncompleteState) -> Self {
        Self {
            cache: state.cache,
            root: state.root,
            end_bound: state.end_state.cursor.width(),
        }
    }
}
impl<'a, G: HasGraphMut + 'a> From<(&'a mut G, InitInterval)>
    for IntervalGraph
{
    fn from((trav, init): (&'a mut G, InitInterval)) -> Self {
        let InitInterval {
            root,
            cache,
            end_bound,
        } = init;
        let ctx = TraceContext { trav, cache };
        let iter = SplitTraceStatesContext::new(ctx, root, end_bound);
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
