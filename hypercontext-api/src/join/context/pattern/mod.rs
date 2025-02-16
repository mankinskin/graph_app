use derivative::Derivative;
use derive_more::derive::{
    Deref,
    DerefMut,
};

use crate::{
    graph::vertex::pattern::id::PatternId,
    join::SplitMap,
    interval::partition::pattern::{
        HasPatternTraceContext,
        PatternTraceContext,
    },
};

pub mod borders;

#[derive(Debug, Deref, DerefMut, Derivative)]
#[derivative(Hash, PartialEq, Eq)]
pub struct PatternJoinContext<'p> {
    #[deref]
    #[deref_mut]
    pub ctx: PatternTraceContext<'p>,
    #[derivative(Hash = "ignore", PartialEq = "ignore")]
    pub splits: &'p SplitMap,
}

impl<'a> HasPatternTraceContext for PatternJoinContext<'a> {
    fn pattern_trace_context<'b>(&'b self) -> PatternTraceContext<'b>
    where
        Self: 'b,
    {
        self.ctx
    }
}

impl<'p> From<PatternJoinContext<'p>> for PatternId {
    fn from(value: PatternJoinContext<'p>) -> Self {
        Self::from(value.ctx)
    }
}
