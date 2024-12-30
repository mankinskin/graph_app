use derivative::Derivative;

use crate::graph::vertex::{
    location::pattern::PatternLocation,
    pattern::Pattern,
    pattern::id::PatternId,
};


//pub trait AsPatternContext<'p> {
//    type PatternCtx<'t>;
//    fn as_pattern_context<'t>(&'t self) -> Self::PatternCtx<'t> where Self: 't, 'p: 't;
//}
//impl<'p> AsPatternContext<'p> for PatternJoinContext<'p> {
//    fn as_pattern_join_context<'t>(&'t self) -> PatternJoinContext<'t> where Self: 't, 'p: 't {
//        *self
//    }
//}

#[derive(Debug, Clone, Copy, Derivative)]
#[derivative(Hash, PartialEq, Eq)]
pub struct PatternTraceContext<'a> {
    pub loc: PatternLocation,
    #[derivative(Hash = "ignore", PartialEq = "ignore")]
    pub pattern: &'a Pattern,
}

impl<'p> From<PatternTraceContext<'p>> for PatternId {
    fn from(value: PatternTraceContext<'p>) -> Self {
        value.loc.id
    }
}

pub trait AsPatternTraceContext: {
    fn as_pattern_trace_context<'a>(&'a self) -> PatternTraceContext<'a>
        where Self: 'a;
}

impl<'a> AsPatternTraceContext for PatternTraceContext<'a> {
    fn as_pattern_trace_context<'b>(&'b self) -> PatternTraceContext<'b>
        where Self: 'b
    {
        *self
    }
}
pub trait ToPatternContext {
    type PatternCtx: AsPatternTraceContext;
    fn to_pattern_context(
        self,
        pattern_id: &PatternId,
    ) -> Self::PatternCtx;
}



pub trait AsPatternContext {
    type PatternCtx<'b>: AsPatternTraceContext where Self: 'b;
    fn as_pattern_context<'b>(
        &'b self,
        pattern_id: &PatternId,
    ) -> Self::PatternCtx<'b>
        where Self: 'b;
}
