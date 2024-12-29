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

pub trait VisitMode<R: RangeRole<Mode = Self>>:
    Debug + Clone + Copy + ModeChildren<R> + for<'a> ModeContext<'a>
{
    type RangeInfo: ModeRangeInfo<R>;
}

pub type RangeInfoOf<R> = <ModeOf<R> as VisitMode<R>>::RangeInfo;

impl<R: RangeRole<Mode = Self>> VisitMode<R> for Trace {
    type RangeInfo = TraceRangeInfo<R>;
}
