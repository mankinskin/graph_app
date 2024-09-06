use crate::join::{
    context::pattern::AsPatternTraceContext,
    partition::info::{
        border::visit::VisitBorders,
        range::{
            role::{
                ModePatternCtxOf,
                RangeRole,
            },
            InnerRangeInfo,
        },
    },
};

pub trait TraceBorders<K: RangeRole>: VisitBorders<K> {
    fn inner_info(
        &self,
        ctx: &ModePatternCtxOf<'_, K>,
    ) -> Option<InnerRangeInfo<K>>;
}

impl<K: RangeRole> TraceBorders<K> for K::Borders {
    fn inner_info(
        &self,
        ctx: &ModePatternCtxOf<'_, K>,
    ) -> Option<InnerRangeInfo<K>> {
        let pctx = ctx.as_pattern_trace_context();
        self.inner_range_offsets(pctx.pattern)
            .map(|offsets| InnerRangeInfo {
                range: self.inner_range(),
                offsets,
            })
    }
}
