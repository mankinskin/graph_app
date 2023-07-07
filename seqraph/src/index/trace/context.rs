use crate::*;

#[derive(Debug, Clone, Copy)]
pub struct TraceContext<'p> {
    pub patterns: &'p ChildPatterns,
    pub index: Child,
}
impl<'p> TraceContext<'p> {
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
pub trait AsTraceContext<'p>: 'p {
    type PatternCtx<'t>: AsPatternTraceContext<'t> where Self: 't, 'p: 't;
    fn as_trace_context<'t>(&'t self) -> TraceContext<'t> where Self: 't, 'p: 't;
    fn pattern_context<'t>(&'t self, pattern_id: PatternId, pattern: &'p Pattern) -> Self::PatternCtx<'t> where Self: 't, 'p: 't;
}
impl<'p> AsTraceContext<'p> for TraceContext<'p> {
    type PatternCtx<'t> = PatternTraceContext<'t> where Self: 't, 'p: 't;
    fn as_trace_context<'t>(&'t self) -> TraceContext<'t> where Self: 't, 'p: 't {
        *self
    }
    fn pattern_context<'t>(&'t self, pattern_id: PatternId, pattern: &'p Pattern) -> Self::PatternCtx<'t> where Self: 't, 'p: 't {
        Self::PatternCtx {
            pattern_id,
            pattern,
        }
    }
}
impl<'p, P: Borrow<JoinContext<'p>> + 'p> AsTraceContext<'p> for P {
    type PatternCtx<'t> = PatternJoinContext<'t> where Self: 't, 'p: 't;
    fn as_trace_context<'t>(&'t self) -> TraceContext<'t> where Self: 't, 'p: 't {
        TraceContext {
            patterns: self.borrow().patterns(),
            index: self.borrow().index,
        }
    }
    fn pattern_context<'t>(&'t self, pattern_id: PatternId, pattern: &'p Pattern) -> Self::PatternCtx<'t> where Self: 't, 'p: 't {
        Self::PatternCtx {
            pattern_id,
            pattern,
            sub_splits: self.borrow().sub_splits,
        }
    }
}
#[derive(Debug, Clone, Copy)]
pub struct PatternTraceContext<'p> {
    pub pattern_id: PatternId,
    pub pattern: &'p Pattern,
}
pub trait AsPatternTraceContext<'p>: 'p {
    fn as_pattern_trace_context<'t>(&'t self) -> PatternTraceContext<'t> where Self: 't, 'p: 't;
}
impl<'p> AsPatternTraceContext<'p> for PatternTraceContext<'p> {
    fn as_pattern_trace_context<'t>(&'t self) -> PatternTraceContext<'t> where Self: 't, 'p: 't {
        *self
    }
}
//impl<'p, P: Borrow<ChildPatterns> + 'p, I: Borrow<Child> + 'p> AsTraceContext<'p> for (P, I) {
//    type PatternCtx<'t> = PatternTraceContext<'t> where Self: 't, 'p: 't;
//    fn as_trace_context<'t>(&'t self) -> TraceContext<'t> where Self: 't, 'p: 't {
//        TraceContext {
//            patterns: self.0.borrow(),
//            index: *self.1.borrow(),
//        }
//    }
//    fn pattern_context<'t>(&'t self, pattern_id: PatternId, pattern: &'p Pattern) -> Self::PatternCtx<'t> where Self: 't, 'p: 't {
//        TraceContext::pattern_context(self.as_trace_context(), pattern_id, pattern)
//    }
//}