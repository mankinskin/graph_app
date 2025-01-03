use std::{hash::Hash, fmt::Debug};

use crate::partition::{context::{AsNodeTraceContext, NodeTraceContext}, info::range::{
    role::{
        ModeOf,
        RangeRole,
    },
    ModeRangeInfo,
    TraceRangeInfo,
}, pattern::{GetPatternContext, GetPatternTraceContext, HasPatternTraceContext, PatternTraceContext}};

use super::role::{Pre, In, Post};

#[derive(Debug, Clone, Copy)]
pub struct Trace;

pub trait ModeInfo<R: RangeRole<Mode = Self>>:
    Debug + Clone + Copy + ModeChildren<R> + ModeContext
{
    type PatternInfo: ModeRangeInfo<R>;
}

pub type PatternInfoOf<R> = <ModeOf<R> as ModeInfo<R>>::PatternInfo;

impl<R: RangeRole<Mode = Self>> ModeInfo<R> for Trace {
    type PatternInfo = TraceRangeInfo<R>;
}

pub trait ModeContext {
    type NodeContext<'a: 'b, 'b>: AsNodeTraceContext
        + GetPatternContext<PatternCtx<'b> = Self::PatternResult<'b>>
        + GetPatternTraceContext  + 'b where Self: 'a;
    type PatternResult<'a>: HasPatternTraceContext + Hash + Eq
        where Self: 'a;
}

impl ModeContext for Trace {
    type NodeContext<'a: 'b, 'b> = NodeTraceContext<'b>;
    type PatternResult<'a> = PatternTraceContext<'a>;
}

pub trait ModeChildren<R: RangeRole> {
    type Result: Clone + Debug;
}

impl<R: RangeRole<Mode = Trace>> ModeChildren<R> for Trace {
    type Result = ();
}
pub trait PreVisitMode: ModeInfo<Pre<Self>> {}

impl PreVisitMode for Trace {}

pub trait PostVisitMode: ModeInfo<Post<Self>> {}

impl PostVisitMode for Trace {}


pub trait InVisitMode: ModeInfo<In<Self>> + PreVisitMode + PostVisitMode {}

impl InVisitMode for Trace {}

