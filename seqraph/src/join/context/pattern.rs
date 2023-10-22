use crate::*;

#[derive(Debug, Deref, DerefMut)]
pub struct PatternJoinContext<'p> {
    #[deref]
    #[deref_mut]
    pub ctx: PatternTraceContext<'p>,
    //pub graph: RwLockWriteGuard<'p, Hypergraph>,
    pub sub_splits: &'p SubSplits,
}
impl<'p> AsPatternTraceContext<'p> for PatternJoinContext<'p> {
    fn as_pattern_trace_context<'t>(&'t self) -> PatternTraceContext<'t> where Self: 't, 'p: 't {
        self.ctx
    }
}
//pub trait AsPatternContext<'p> {
//    type PatternCtx<'t>;
//    fn as_pattern_context<'t>(&'t self) -> Self::PatternCtx<'t> where Self: 't, 'p: 't;
//}
//impl<'p> AsPatternContext<'p> for PatternJoinContext<'p> {
//    fn as_pattern_join_context<'t>(&'t self) -> PatternJoinContext<'t> where Self: 't, 'p: 't {
//        *self
//    }
//}

#[derive(Debug, Clone, Copy)]
pub struct PatternTraceContext<'p> {
    pub loc: PatternLocation,
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