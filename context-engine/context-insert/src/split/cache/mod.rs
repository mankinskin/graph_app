use crate::split::{
    cache::vertex::SplitVertexCache,
    trace::{
        SplitTraceState,
        states::context::SplitTraceStatesContext,
    },
    vertex::output::RootMode,
};
use context_trace::{
    HashMap,
    graph::vertex::{
        VertexIndex,
        child::Child,
        has_vertex_index::HasVertexIndex,
    },
    trace::{
        has_graph::HasGraph,
        node::NodeTraceContext,
    },
};
use derive_more::derive::{
    Deref,
    DerefMut,
};
use derive_new::new;
use std::fmt::Debug;

pub mod vertex;

pub mod leaves;
pub mod position;

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
        ctx: &mut SplitTraceStatesContext<G>,
        nodes: I,
    ) {
        for c in nodes {
            let new = self.augment_node(&ctx.trav, c);
            // todo: force order
            ctx.states.queue.extend(new.into_iter());
        }
    }
}
