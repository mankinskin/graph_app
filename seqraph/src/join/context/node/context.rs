use crate::*;

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

pub trait AsNodeTraceContext<'p>: 'p {
    fn as_trace_context<'t>(&'t self) -> NodeTraceContext<'t> where Self: 't, 'p: 't;
}
impl<'p> AsNodeTraceContext<'p> for NodeTraceContext<'p> {
    fn as_trace_context<'t>(&'t self) -> NodeTraceContext<'t> where Self: 't, 'p: 't {
        *self
    }
}
#[derive(Debug, Deref, DerefMut)]
pub struct NodeJoinContext<'p, S: HasPosSplits + 'p = SplitVertexCache> {
    #[deref]
    #[deref_mut]
    pub ctx: JoinContext<'p>,
    pub index: Child,
    pub pos_splits: &'p PosSplits<S>,
}
impl<'p, S: HasPosSplits + 'p> AsNodeTraceContext<'p> for NodeJoinContext<'p, S> {
    fn as_trace_context<'t>(&'t self) -> NodeTraceContext<'t> where Self: 't, 'p: 't {
        NodeTraceContext {
            patterns: self.borrow().patterns(),
            index: self.borrow().index,
        }
    }
}

pub trait ToPatternContext<'p> {
    type PatternCtx<'a>: AsPatternTraceContext<'p> where Self: 'a, 'a: 'p;
    fn to_pattern_context<'t>(self,  pattern_id: &PatternId) -> Self::PatternCtx<'t> where Self: 't, 't: 'p;
}
impl<'p, SP: HasPosSplits + 'p> AsPatternContext<'p> for NodeJoinContext<'p, SP> {
    type PatternCtx<'a> = PatternJoinContext<'a> where Self: 'a, 'a: 'p;
    fn as_pattern_context<'t>(&'t self, pattern_id: &PatternId) -> Self::PatternCtx<'t> where Self: 't, 't: 'p {

        let ctx = PatternTraceContext {
            loc: self.index.to_pattern_location(*pattern_id),
            pattern: self.as_trace_context().patterns.get(pattern_id).unwrap(),
        };
        PatternJoinContext {
            //graph: self.graph,
            ctx,
            sub_splits: self.borrow().sub_splits,
        }
    }
}
pub trait AsPatternContext<'p> {
    type PatternCtx<'a>: AsPatternTraceContext<'p> where Self: 'a, 'a: 'p;
    fn as_pattern_context<'t>(&'t self,  pattern_id: &PatternId) -> Self::PatternCtx<'t> where Self: 't, 't: 'p;
}
impl<'p> AsPatternContext<'p> for NodeTraceContext<'p> {
    type PatternCtx<'a> = PatternTraceContext<'p> where Self: 'a, 'a: 'p;
    fn as_pattern_context<'t>(&'t self, pattern_id: &PatternId) -> Self::PatternCtx<'t> where Self: 't, 't: 'p {
        PatternTraceContext {
            loc: self.index.to_pattern_location(*pattern_id),
            pattern: self.as_trace_context().patterns.get(pattern_id).unwrap(),
        }
    }
}
impl<'p, SP: HasPosSplits + 'p> NodeJoinContext<'p, SP> {
    pub fn new(
        ctx: JoinContext<'p>,
        index: Child,
        pos_splits: &'p SP,
    ) -> Self {
        Self {
            ctx,
            index,
            pos_splits: pos_splits.pos_splits(),
        }
    }
    pub fn patterns(&self) -> &ChildPatterns {
        self.ctx.graph.expect_child_patterns(self.index)
    }
}