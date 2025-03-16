pub mod states;
use derive_more::derive::{
    Deref,
    DerefMut,
};

use super::vertex::VertexSplitContext;
use crate::{
    graph::vertex::{
        child::Child,
        has_vertex_index::HasVertexIndex,
    },
    traversal::{
        split::vertex::output::NodeType,
        trace::context::TraceContext,
        traversable::Traversable,
    },
};
#[derive(Debug, Deref, DerefMut)]
pub struct SplitTraceContext<Trav: Traversable> {
    pub root: Child,
    pub end_bound: usize,

    #[deref]
    #[deref_mut]
    pub ctx: TraceContext<Trav>,
}

impl<Trav: Traversable> SplitTraceContext<Trav> {
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
            .map(|ctx| ctx.complete_splits::<_, N>(&self.trav, self.end_bound.into()))
            .unwrap_or_default()
    }
}
