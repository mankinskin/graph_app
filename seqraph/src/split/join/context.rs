use crate::*;


#[derive(Debug)]
pub struct JoinContext<'p, S: HasSplitPos = SplitVertexCache> {
    pub graph: RwLockWriteGuard<'p, Hypergraph>,
    pub index: Child,
    pub split_pos: &'p SplitPos<S>,
    pub sub_splits: &'p SubSplits,
}
impl<'p, SP: HasSplitPos> JoinContext<'p, SP> {
    pub fn new<SS: HasSubSplits>(
        graph: RwLockWriteGuard<'p, Hypergraph>,
        index: Child,
        split_pos: &'p SP,
        sub_splits: &'p SS,
    ) -> Self {
        Self {
            graph,
            index,
            split_pos: split_pos.split_pos(),
            sub_splits: sub_splits.sub_splits(),
        }
    }
    pub fn patterns(&self) -> &ChildPatterns {
        self.graph.expect_child_patterns(self.index)
    }
}

pub trait AsJoinContext<'p>: AsTraceContext<'p, PatternCtx<'p> = PatternJoinContext<'p>> {
    fn as_join_context<'t>(self) -> JoinContext<'t> where Self: 't, 'p: 't;
}
impl<'p> AsJoinContext<'p> for JoinContext<'p> {
    fn as_join_context<'t>(self) -> JoinContext<'t> where Self: 't, 'p: 't {
        self
    }

}
#[derive(Debug, Clone, Copy)]
pub struct PatternJoinContext<'p> {
    pub pattern_id: PatternId,
    pub pattern: &'p Pattern,
    pub sub_splits: &'p SubSplits,
}
impl<'p> AsPatternTraceContext<'p> for PatternJoinContext<'p> {
    fn as_pattern_trace_context<'t>(&'t self) -> PatternTraceContext<'t> where Self: 't, 'p: 't {
        PatternTraceContext {
            pattern_id: self.pattern_id,
            pattern: self.pattern,
        }
    }
}
pub trait AsPatternJoinContext<'p>: AsPatternTraceContext<'p> {
    fn as_pattern_join_context<'t>(&'t self) -> PatternJoinContext<'t> where Self: 't, 'p: 't;
}
impl<'p> AsPatternJoinContext<'p> for PatternJoinContext<'p> {
    fn as_pattern_join_context<'t>(&'t self) -> PatternJoinContext<'t> where Self: 't, 'p: 't {
        *self
    }
}
//impl<'p, C: AsJoinContext<'p>> AsJoinContext<'p> for &'p C
//    where &'p C: AsTraceContext<'p, PatternCtx<'p> = PatternJoinContext<'p>>
//{
//    fn as_join_context<'t>(self) -> JoinContext<'t> where Self: 't, 'p: 't {
//        (*self).as_join_context()
//    }
//}
//impl<'p, C: AsJoinContext<'p>> AsJoinContext<'p> for &'p mut C
//    where &'p mut C: AsTraceContext<'p, PatternCtx<'p> = PatternJoinContext<'p>>
//{
//    fn as_join_context<'t>(self) -> JoinContext<'t> where Self: 't, 'p: 't {
//        (*self).as_join_context()
//    }
//}