use std::fmt::Debug;

use crate::split::{
    cache::{
        position::{
            PosKey,
            SplitPositionCache,
        },
        vertex::SplitVertexCache,
    },
    context::SplitCacheContext,
    trace::{
        SplitTraceState,
        states::{
            SplitStates,
            SplitTraceStateContext,
        },
    },
    vertex::output::RootMode,
};
use context_search::traversal::result::IncompleteState;
use context_trace::{
    HashMap,
    graph::vertex::{
        VertexIndex,
        child::Child,
        has_vertex_index::HasVertexIndex,
        wide::Wide,
    },
    trace::{
        TraceContext,
        cache::TraceCache,
        has_graph::{
            HasGraph,
            HasGraphMut,
        },
        node::NodeTraceContext,
    },
};
use derive_more::derive::{
    Deref,
    DerefMut,
};
use derive_new::new;
pub mod partition;

#[derive(Debug, Deref, DerefMut, new)]
pub struct SplitCache {
    pub root_mode: RootMode,
    #[deref]
    #[deref_mut]
    entries: HashMap<VertexIndex, SplitVertexCache>,
}
impl SplitCache {
    pub fn augment_node(
        &mut self,
        trav: impl HasGraph,
        index: Child,
    ) -> Vec<SplitTraceState> {
        let graph = trav.graph();
        let ctx = NodeTraceContext::new(&graph, index);
        self.get_mut(&index.vertex_index())
            .unwrap()
            .augment_node(ctx)
    }
    /// complete inner range offsets for root
    pub fn augment_root(
        &mut self,
        trav: impl HasGraph,
        root: Child,
    ) -> Vec<SplitTraceState> {
        let graph = trav.graph();
        let ctx = NodeTraceContext::new(&graph, root);
        let index = root.vertex_index();
        let root_mode = self.root_mode;
        self.get_mut(&index).unwrap().augment_root(ctx, root_mode)
    }
    pub fn augment_nodes<G: HasGraph, I: IntoIterator<Item = Child>>(
        &mut self,
        ctx: &mut SplitTraceStateContext<G>,
        nodes: I,
    ) {
        for c in nodes {
            let new = self.augment_node(&ctx.trav, c);
            // todo: force order
            ctx.states.queue.extend(new.into_iter());
        }
    }
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
