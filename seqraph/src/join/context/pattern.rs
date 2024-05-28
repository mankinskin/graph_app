use derivative::Derivative;
use derive_more::{
    Deref,
    DerefMut,
};

use crate::{
    join::partition::splits::SubSplits,
    vertex::{
        location::pattern::PatternLocation,
        pattern::Pattern,
        PatternId,
    },
};

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
