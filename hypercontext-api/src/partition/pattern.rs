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
pub struct PatternTraceContext<'p> {
    pub loc: PatternLocation,
    #[derivative(Hash = "ignore", PartialEq = "ignore")]
    pub pattern: &'p Pattern,
}

impl<'p> From<PatternTraceContext<'p>> for PatternId {
    fn from(value: PatternTraceContext<'p>) -> Self {
        value.loc.id
    }
}

pub trait AsPatternTraceContext<'p>: 'p {
    fn as_pattern_trace_context<'t>(&'t self) -> PatternTraceContext<'t>
    where
        Self: 't,
        'p: 't;
}

impl<'p> AsPatternTraceContext<'p> for PatternTraceContext<'p> {
    fn as_pattern_trace_context<'t>(&'t self) -> PatternTraceContext<'t>
    where
        Self: 't,
        'p: 't,
    {
        *self
    }
}
pub trait ToPatternContext<'p> {
    type PatternCtx<'a>: AsPatternTraceContext<'p>
    where
        Self: 'a,
        'a: 'p;
    fn to_pattern_context<'t>(
        self,
        pattern_id: &PatternId,
    ) -> Self::PatternCtx<'t>
    where
        Self: 't,
        't: 'p;
}



pub trait AsPatternContext<'p> {
    type PatternCtx<'a>: AsPatternTraceContext<'p>
    where
        Self: 'a,
        'a: 'p;
    fn as_pattern_context<'t>(
        &'p self,
        pattern_id: &PatternId,
    ) -> Self::PatternCtx<'t>
    where
        Self: 't,
        't: 'p;
}
