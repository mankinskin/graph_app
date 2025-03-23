use crate::graph::{
    getters::vertex::VertexSet,
    kind::GraphKind,
    vertex::{
        child::Child,
        pattern::id::PatternId,
        ChildPatterns,
    },
    Hypergraph,
};

use super::pattern::{
    GetPatternContext,
    GetPatternTraceContext,
    PatternTraceContext,
};

#[derive(Debug, Clone, Copy)]
pub struct NodeTraceContext<'p> {
    pub patterns: &'p ChildPatterns,
    pub index: Child,
}

impl<'p> NodeTraceContext<'p> {
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

impl GetPatternContext for NodeTraceContext<'_> {
    type PatternCtx<'b>
        = PatternTraceContext<'b>
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
impl GetPatternTraceContext for NodeTraceContext<'_> {
    fn get_pattern_trace_context<'b>(
        &'b self,
        pattern_id: &PatternId,
    ) -> PatternTraceContext<'b>
    where
        Self: 'b,
    {
        PatternTraceContext {
            loc: self.index.to_pattern_location(*pattern_id),
            pattern: self.patterns.get(pattern_id).unwrap(),
        }
    }
}

pub trait AsNodeTraceContext {
    fn as_trace_context<'a>(&'a self) -> NodeTraceContext<'a>
    where
        Self: 'a;
}

impl AsNodeTraceContext for NodeTraceContext<'_> {
    fn as_trace_context<'b>(&'b self) -> NodeTraceContext<'b>
    where
        Self: 'b,
    {
        *self
    }
}
