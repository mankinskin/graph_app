use crate::graph::{
    Hypergraph,
    getters::vertex::VertexSet,
    kind::GraphKind,
    vertex::{
        ChildPatterns,
        child::Child,
        pattern::id::PatternId,
    },
};

use super::pattern::{
    GetPatternCtx,
    GetPatternTraceCtx,
    PatternTraceCtx,
};

#[derive(Debug, Clone, Copy)]
pub struct NodeTraceCtx<'p> {
    pub patterns: &'p ChildPatterns,
    pub index: Child,
}

impl<'p> NodeTraceCtx<'p> {
    pub fn new<K: GraphKind>(
        graph: &'p Hypergraph<K>,
        index: Child,
    ) -> Self {
        Self {
            patterns: &graph.expect_vertex(index).children,
            index,
        }
    }
}

impl GetPatternCtx for NodeTraceCtx<'_> {
    type PatternCtx<'b>
        = PatternTraceCtx<'b>
    where
        Self: 'b;
    fn get_pattern_context<'b>(
        &'b self,
        pattern_id: &PatternId,
    ) -> Self::PatternCtx<'b>
    where
        Self: 'b,
    {
        self.get_pattern_trace_context(pattern_id)
    }
}
impl GetPatternTraceCtx for NodeTraceCtx<'_> {
    fn get_pattern_trace_context<'b>(
        &'b self,
        pattern_id: &PatternId,
    ) -> PatternTraceCtx<'b>
    where
        Self: 'b,
    {
        PatternTraceCtx {
            loc: self.index.to_pattern_location(*pattern_id),
            pattern: self.patterns.get(pattern_id).unwrap(),
        }
    }
}

pub trait AsNodeTraceCtx {
    fn as_trace_context<'a>(&'a self) -> NodeTraceCtx<'a>
    where
        Self: 'a;
}

impl AsNodeTraceCtx for NodeTraceCtx<'_> {
    fn as_trace_context<'b>(&'b self) -> NodeTraceCtx<'b>
    where
        Self: 'b,
    {
        *self
    }
}
