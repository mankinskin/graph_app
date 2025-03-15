use derivative::Derivative;

use crate::graph::vertex::{
    location::pattern::PatternLocation,
    pattern::Pattern,
    pattern::id::PatternId,
};


//pub trait GetPatternContext<'p> {
//    type PatternCtx<'t>;
//    fn get_pattern_context<'t>(&'t self) -> Self::PatternCtx<'t> where Self: 't, 'p: 't;
//}
//impl<'p> GetPatternContext<'p> for PatternJoinContext<'p> {
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

pub trait HasPatternTraceContext {
    fn pattern_trace_context<'a>(&'a self,
    ) -> PatternTraceContext<'a>
        where Self: 'a;
}
impl HasPatternTraceContext for PatternTraceContext<'_> {
    fn pattern_trace_context<'a>(&'a self) -> PatternTraceContext<'a>
        where Self: 'a {
        *self
    }
}
pub trait GetPatternTraceContext {
    fn get_pattern_trace_context<'b>(
        &'b self,
        pattern_id: &PatternId,
    ) -> PatternTraceContext<'b>
        where Self: 'b;
}
pub trait GetPatternContext {
    type PatternCtx<'b>: HasPatternTraceContext where Self: 'b;
    fn get_pattern_context<'b>(
        &'b self,
        pattern_id: &PatternId,
    ) -> Self::PatternCtx<'b>
        where Self: 'b;
}

//impl<'b, T: GetPatternContext<PatternCtx<'b> = PatternTraceContext<'b>>> GetPatternTraceContext<'b> for T
//{
//    fn get_pattern_trace_context(&'b self,
//        pattern_id: &PatternId,
//    ) -> PatternTraceContext<'b>
//        where Self: 'b
//    {
//        self.get_pattern_context(pattern_id)
//    }
//}

//pub trait ToPatternContext {
//    type PatternCtx: GetPatternTraceContext;
//    fn to_pattern_context(
//        self,
//        pattern_id: &PatternId,
//    ) -> Self::PatternCtx;
//}

