use crate::*;

pub trait TraceBorders<K: RangeRole>: VisitBorders<K>
{
    fn inner_info<'a>(
        &self,
        ctx: &ModePatternCtxOf<'a, K>,
    ) -> Option<InnerRangeInfo<K>>;
}
impl<K: RangeRole> TraceBorders<K> for K::Borders {
    fn inner_info<'a>(
        &self,
        ctx: &ModePatternCtxOf<'a, K>,
    ) -> Option<InnerRangeInfo<K>> {
        let pctx = ctx.as_pattern_trace_context();
        self.inner_range_offsets(pctx.pattern)
            .map(|offsets|
                InnerRangeInfo {
                    range: self.inner_range(),
                    offsets,
                }
            )
    }
}