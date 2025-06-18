use std::fmt::Debug;

use mode::{
    PatternInfoOf,
    Trace,
};
use role::{
    BordersOf,
    ModePatternCtxOf,
    RangeRole,
};

use context_trace::{
    graph::vertex::{
        child::Child,
        pattern::id::PatternId,
    },
    trace::pattern::HasPatternTraceCtx,
};

use super::border::{
    trace::TraceBorders,
    visit::VisitBorders,
};

pub mod role;

pub mod children;
pub mod mode;
pub mod splits;

#[derive(Debug)]
pub struct PatternRangeInfo<R: RangeRole> {
    pub pattern_id: PatternId,
    pub info: PatternInfoOf<R>,
}

impl<R: RangeRole> From<PatternRangeInfo<R>> for (PatternId, PatternInfoOf<R>) {
    fn from(val: PatternRangeInfo<R>) -> Self {
        (val.pattern_id, val.info)
    }
}

pub trait ModeRangeInfo<R: RangeRole>: Debug {
    fn info_pattern_range(
        borders: BordersOf<R>,
        ctx: &ModePatternCtxOf<R>,
    ) -> Result<PatternRangeInfo<R>, Child>;
}

impl<R: RangeRole<Mode = Trace>> ModeRangeInfo<R> for TraceRangeInfo<R> {
    fn info_pattern_range(
        borders: BordersOf<R>,
        ctx: &ModePatternCtxOf<R>,
    ) -> Result<PatternRangeInfo<R>, Child> {
        let range = borders.outer_range();
        let inner = borders.inner_info(ctx);
        let (pat, pid) = {
            let ctx = ctx.pattern_trace_context();
            let pat = ctx.pattern.get(range.clone()).unwrap();
            (pat, ctx.loc.id)
        };
        if pat.len() != 1 {
            Ok(PatternRangeInfo {
                pattern_id: pid,
                info: TraceRangeInfo { inner_range: inner },
            })
        } else {
            Err(pat[0])
        }
    }
}

#[derive(Debug, Clone)]
pub struct InnerRangeInfo<R: RangeRole> {
    pub range: R::Range,
    pub offsets: R::Offsets,
}

#[derive(Debug, Clone)]
pub struct TraceRangeInfo<R: RangeRole<Mode = Trace>> {
    pub inner_range: Option<InnerRangeInfo<R>>,
}
