use derivative::Derivative;
use derive_more::derive::{Deref, DerefMut};

use crate::{graph::vertex::pattern::id::PatternId, partition::{pattern::{AsPatternTraceContext, PatternTraceContext}, splits::SubSplits}};


#[derive(Debug, Deref, DerefMut, Derivative)]
#[derivative(Hash, PartialEq, Eq)]
pub struct PatternJoinContext<'p> {
    #[deref]
    #[deref_mut]
    pub ctx: PatternTraceContext<'p>,
    //pub graph: RwLockWriteGuard<'p, Hypergraph>,
    #[derivative(Hash = "ignore", PartialEq = "ignore")]
    pub sub_splits: &'p SubSplits,
}

impl<'p> AsPatternTraceContext<'p> for PatternJoinContext<'p> {
    fn as_pattern_trace_context<'t>(&'t self) -> PatternTraceContext<'t>
    where
        Self: 't,
        'p: 't,
    {
        self.ctx
    }
}

impl<'p> From<PatternJoinContext<'p>> for PatternId {
    fn from(value: PatternJoinContext<'p>) -> Self {
        Self::from(value.ctx)
    }
}