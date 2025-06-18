use std::{
    fmt::Debug,
    hash::Hash,
};

use crate::interval::partition::info::range::{
    ModeRangeInfo,
    TraceRangeInfo,
    role::{
        ModeOf,
        RangeRole,
    },
};
use context_trace::trace::{
    node::{
        AsNodeTraceCtx,
        NodeTraceCtx,
    },
    pattern::{
        GetPatternCtx,
        GetPatternTraceCtx,
        HasPatternTraceCtx,
        PatternTraceCtx,
    },
};

use super::role::{
    In,
    Post,
    Pre,
};

#[derive(Debug, Clone, Copy)]
pub struct Trace;

pub trait ModeInfo<R: RangeRole<Mode = Self>>:
    Debug + Clone + Copy + ModeChildren<R> + ModeCtx
{
    type PatternInfo: ModeRangeInfo<R>;
}

pub type PatternInfoOf<R> = <ModeOf<R> as ModeInfo<R>>::PatternInfo;

impl<R: RangeRole<Mode = Self>> ModeInfo<R> for Trace {
    type PatternInfo = TraceRangeInfo<R>;
}

pub trait ModeCtx {
    type NodeCtx<'a: 'b, 'b>: AsNodeTraceCtx
        + GetPatternCtx<PatternCtx<'b> = Self::PatternResult<'b>>
        + GetPatternTraceCtx
        + 'b
    where
        Self: 'a;
    type PatternResult<'a>: HasPatternTraceCtx + Hash + Eq
    where
        Self: 'a;
}

impl ModeCtx for Trace {
    type NodeCtx<'a: 'b, 'b> = NodeTraceCtx<'b>;
    type PatternResult<'a> = PatternTraceCtx<'a>;
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

pub trait InVisitMode:
    ModeInfo<In<Self>> + PreVisitMode + PostVisitMode
{
}

impl InVisitMode for Trace {}
