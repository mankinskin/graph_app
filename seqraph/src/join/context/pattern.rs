use derivative::Derivative;
use derive_more::derive::{Deref, DerefMut};

use hypercontext_api::{graph::vertex::pattern::id::PatternId, partition::{pattern::{HasPatternTraceContext, PatternTraceContext}, splits::SubSplits}};



#[derive(Debug, Deref, DerefMut, Derivative)]
#[derivative(Hash, PartialEq, Eq)]
pub struct PatternJoinContext<'p> {
    #[deref]
    #[deref_mut]
    pub ctx: PatternTraceContext<'p>,
    //pub graph: RwLockWriteGuard<'p, Hypergraph>,
    #[derivative(Hash = "ignore", PartialEq = "ignore")]
    pub sub_splits: &'p SubSplits,
    //#[derivative(Hash = "ignore", PartialEq = "ignore")]
    //pub split_cache: &'p SplitCache,
}

impl<'a> HasPatternTraceContext for PatternJoinContext<'a> {
    fn pattern_trace_context<'b>(&'b self) -> PatternTraceContext<'b>
        where Self: 'b
    {
        self.ctx
    }
}

impl<'p> From<PatternJoinContext<'p>> for PatternId {
    fn from(value: PatternJoinContext<'p>) -> Self {
        Self::from(value.ctx)
    }
}
