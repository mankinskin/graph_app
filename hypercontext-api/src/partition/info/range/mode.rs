use std::fmt::Debug;

use crate::partition::info::range::{
    role::{
        ModeChildren,
        ModeContext,
        ModeOf,
        RangeRole,
        Trace,
    },
    ModeRangeInfo,
    TraceRangeInfo,
};

pub trait ModeInfo<R: RangeRole<Mode = Self>>:
    Debug + Clone + Copy + ModeChildren<R> + ModeContext
{
    type PatternInfo: ModeRangeInfo<R>;
}

pub type PatternInfoOf<R> = <ModeOf<R> as ModeInfo<R>>::PatternInfo;

impl<R: RangeRole<Mode = Self>> ModeInfo<R> for Trace {
    type PatternInfo = TraceRangeInfo<R>;
}
