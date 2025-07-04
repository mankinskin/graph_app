pub mod states;

use derive_more::derive::{
    Deref,
    DerefMut,
};

use super::{
    cache::position::PosKey,
    vertex::VertexSplitCtx,
};
use crate::split::vertex::output::NodeType;
use context_trace::{
    graph::vertex::{
        child::Child,
        has_vertex_index::HasVertexIndex,
    },
    trace::{
        TraceCtx,
        cache::position::Offset,
        has_graph::HasGraph,
    },
};

#[derive(Debug, Clone)]
pub struct SplitTraceState {
    pub index: Child,
    pub offset: Offset,
    pub prev: PosKey,
}

#[derive(Debug, Deref, DerefMut)]
pub struct SplitTraceCtx<G: HasGraph> {
    pub root: Child,
    pub end_bound: usize,

    #[deref]
    #[deref_mut]
    pub ctx: TraceCtx<G>,
}

impl<G: HasGraph> SplitTraceCtx<G> {
    pub fn get_node<'a, N: NodeType>(
        &'a self,
        index: &Child,
    ) -> Option<VertexSplitCtx<'a>> {
        self.cache
            .entries
            .get(&index.vertex_index())
            .map(|vcache| VertexSplitCtx::new(vcache))
    }
    pub fn completed_splits<N: NodeType>(
        &self,
        index: &Child,
    ) -> N::CompleteSplitOutput {
        self.get_node::<N>(index)
            .map(|ctx| {
                ctx.complete_splits::<_, N>(&self.trav, self.end_bound.into())
            })
            .unwrap_or_default()
    }
}
