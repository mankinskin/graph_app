use crate::interval::partition::info::{
    border::visit::VisitBorders,
    range::{
        InnerRangeInfo,
        role::{
            ModePatternCtxOf,
            RangeRole,
        },
    },
};
use context_tracetrace::pattern::HasPatternTraceContext;

pub trait TraceBorders<R: RangeRole>: VisitBorders<R>
{
    fn inner_info(
        &self,
        ctx: &ModePatternCtxOf<'_, R>,
    ) -> Option<InnerRangeInfo<R>>;
}

impl<R: RangeRole> TraceBorders<R> for R::Borders
{
    fn inner_info(
        &self,
        ctx: &ModePatternCtxOf<'_, R>,
    ) -> Option<InnerRangeInfo<R>>
    {
        let pctx = ctx.pattern_trace_context();
        self.inner_range_offsets(pctx.pattern).map(move |offsets| {
            InnerRangeInfo {
                range: self.inner_range(),
                offsets,
            }
        })
    }
}
