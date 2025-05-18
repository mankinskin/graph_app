use derivative::Derivative;

use crate::graph::vertex::{
    location::pattern::PatternLocation,
    pattern::{
        Pattern,
        id::PatternId,
    },
};

#[derive(Debug, Clone, Derivative)]
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
    fn pattern_trace_context<'a>(&'a self) -> PatternTraceContext<'a>
    where
        Self: 'a;
}
impl HasPatternTraceContext for PatternTraceContext<'_> {
    fn pattern_trace_context<'a>(&'a self) -> PatternTraceContext<'a>
    where
        Self: 'a,
    {
        self.clone()
    }
}
pub trait GetPatternTraceContext {
    fn get_pattern_trace_context<'b>(
        &'b self,
        pattern_id: &PatternId,
    ) -> PatternTraceContext<'b>
    where
        Self: 'b;
}
pub trait GetPatternContext {
    type PatternCtx<'b>: HasPatternTraceContext
    where
        Self: 'b;
    fn get_pattern_context<'b>(
        &'b self,
        pattern_id: &PatternId,
    ) -> Self::PatternCtx<'b>
    where
        Self: 'b;
}
