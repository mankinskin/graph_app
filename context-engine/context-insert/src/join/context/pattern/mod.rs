use derivative::Derivative;
use derive_more::derive::{
    Deref,
    DerefMut,
};

use crate::split::SplitMap;
use context_trace::{
    graph::vertex::pattern::id::PatternId,
    trace::pattern::{
        HasPatternTraceCtx,
        PatternTraceCtx,
    },
};

pub mod borders;

#[derive(Debug, Deref, DerefMut, Derivative)]
#[derivative(Hash, PartialEq, Eq)]
pub struct PatternJoinCtx<'p> {
    #[deref]
    #[deref_mut]
    pub ctx: PatternTraceCtx<'p>,
    #[derivative(Hash = "ignore", PartialEq = "ignore")]
    pub splits: &'p SplitMap,
}

impl<'a> HasPatternTraceCtx for PatternJoinCtx<'a> {
    fn pattern_trace_context<'b>(&'b self) -> PatternTraceCtx<'b>
    where
        Self: 'b,
    {
        self.ctx.clone()
    }
}

impl<'p> From<PatternJoinCtx<'p>> for PatternId {
    fn from(value: PatternJoinCtx<'p>) -> Self {
        Self::from(value.ctx)
    }
}
