use crate::*;

pub trait TraceBorders<'a, K: RangeRole<Borders<'a> = Self>>: VisitBorders<'a, K> {
    fn inner_info(
        &self,
        ctx: &ModePatternCtxOf<'a, K>,
    ) -> Option<InnerRangeInfo<K>>;

    fn info_pattern_range(
        self,
        ctx: &ModePatternCtxOf<'a, K>,
    ) -> Result<PatternRangeInfo<K>, Child> {
        let perfect = self.perfect();
        let range = self.outer_range();
        let offsets = self.offsets();
        let inner = self.inner_info(ctx);
        let (delta, pat, pid) = {
            let ctx = ctx.as_pattern_trace_context();
            let delta = inner.as_ref()
                .and_then(|inner| {
                    let inner_pat = ctx.pattern.get(inner.range.clone()).unwrap();
                    (inner_pat.len() != 1)
                        .then(|| inner_pat.len().saturating_sub(1))
                })
                .unwrap_or(0);
            let pat = ctx.pattern.get(range.clone()).unwrap();
            (delta, pat, ctx.loc.id)
        };
        if pat.len() != 1 || !perfect.all_perfect() {
            let children = ModeOf::<K>::get_child_splits(&self, ctx);
            Ok(PatternRangeInfo {
                pattern_id: pid,
                info: RangeInfo {
                    inner_range: inner,
                    delta,
                    offsets,
                    range,
                    children,
                },
            })
        } else {
            Err(pat[0])
        }

    }
}
impl<'a, K: RangeRole> TraceBorders<'a, K> for K::Borders<'a> {
    fn inner_info(
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