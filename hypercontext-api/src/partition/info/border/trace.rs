use crate::partition::{info::{
        border::visit::VisitBorders,
        range::{
            role::{
                ModePatternCtxOf,
                RangeRole,
            },
            InnerRangeInfo,
        },
    }, pattern::AsPatternTraceContext};

pub trait TraceBorders<R: RangeRole>: VisitBorders<R> {
    fn inner_info(
        &self,
        ctx: &ModePatternCtxOf<'_, R>,
    ) -> Option<InnerRangeInfo<R>>;
}

impl<R: RangeRole> TraceBorders<R> for R::Borders {
    fn inner_info(
        &self,
        ctx: &ModePatternCtxOf<'_, R>,
    ) -> Option<InnerRangeInfo<R>> {
        let pctx = ctx.as_pattern_trace_context();
        self.inner_range_offsets(pctx.pattern)
            .map(move |offsets| InnerRangeInfo {
                range: self.inner_range(),
                offsets,
            })
    }
}
