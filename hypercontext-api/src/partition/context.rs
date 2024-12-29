use std::sync::RwLockWriteGuard;

use crate::graph::{vertex::{child::Child, pattern::id::PatternId, ChildPatterns}, Hypergraph};

use super::pattern::{AsPatternContext, PatternTraceContext};


#[derive(Debug, Clone, Copy)]
pub struct NodeTraceContext<'p> {
    pub patterns: &'p ChildPatterns,
    pub index: Child,
}

impl<'p> NodeTraceContext<'p> {
    pub fn new(
        graph: &'p RwLockWriteGuard<'p, Hypergraph>,
        index: Child,
    ) -> Self {
        Self {
            patterns: graph.expect_child_patterns(index),
            index,
        }
    }
}

impl<'p> AsPatternContext<'p> for NodeTraceContext<'p> {
    type PatternCtx<'a>
        = PatternTraceContext<'p>
    where
        Self: 'a,
        'a: 'p;
    fn as_pattern_context<'t>(
        &'p self,
        pattern_id: &PatternId,
    ) -> Self::PatternCtx<'t>
    where
        Self: 't,
        't: 'p,
    {
        PatternTraceContext {
            loc: self.index.to_pattern_location(*pattern_id),
            pattern: self.as_trace_context().patterns.get(pattern_id).unwrap(),
        }
    }
}


pub trait AsNodeTraceContext<'p>: 'p {
    fn as_trace_context<'t>(&'t self) -> NodeTraceContext<'t>
    where
        Self: 't,
        'p: 't;
}

impl<'p> AsNodeTraceContext<'p> for NodeTraceContext<'p> {
    fn as_trace_context<'t>(&'t self) -> NodeTraceContext<'t>
    where
        Self: 't,
        'p: 't,
    {
        *self
    }
}