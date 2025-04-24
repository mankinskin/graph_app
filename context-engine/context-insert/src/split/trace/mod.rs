pub mod states;

use derive_more::derive::{
    Deref,
    DerefMut,
};

use super::{
    cache::position::PosKey,
    vertex::VertexSplitContext,
};
use crate::split::vertex::output::NodeType;
use context_trace{
    graph::vertex::{
        child::Child,
        has_vertex_index::HasVertexIndex,
    },
    trace::{
        TraceContext,
        cache::entry::position::Offset,
    },
    traversal::has_graph::HasGraph,
};

#[derive(Debug, Clone)]
pub struct SplitTraceState {
    pub index: Child,
    pub offset: Offset,
    pub prev: PosKey,
}

#[derive(Debug, Deref, DerefMut)]
pub struct SplitTraceContext<G: HasGraph> {
    pub root: Child,
    pub end_bound: usize,

    #[deref]
    #[deref_mut]
    pub ctx: TraceContext<G>,
}

impl<G: HasGraph> SplitTraceContext<G> {
    pub fn get_node<'a, N: NodeType>(
        &'a self,
        index: &Child,
    ) -> Option<VertexSplitContext<'a>> {
        self.cache
            .entries
            .get(&index.vertex_index())
            .map(|vcache| VertexSplitContext::new(vcache))
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
