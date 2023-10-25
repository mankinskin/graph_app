use crate::*;
pub struct PartitionBorders<K: RangeRole, C: PartitionBorderKey = PatternId> {
    pub borders: HashMap<C, K::Borders>,
    pub perfect: K::Perfect,
}
pub type PatternCtxs<'t, K> = HashMap<PatternId, ModePatternCtxOf<'t, K>>;
pub trait PartitionBorderKey: Hash + Eq {}
impl<T: Hash + Eq> PartitionBorderKey for T {}


pub trait VisitPartition<K: RangeRole>: Sized + Clone + AsPartition<K> {
    fn info_borders<'t>(
        &self,
        ctx: &PatternTraceContext,
    ) -> K::Borders {
        let part = self.clone().as_partition();
        // todo detect if prev offset is in same index (to use inner partition as result)
        let pctx = ctx.as_pattern_trace_context();
        let splits = part.offsets.get(&pctx.loc.id).unwrap();

        K::Borders::info_border(pctx.pattern, &splits)
    }

    fn pattern_ctxs<'t>(
        &self,
        ctx: &'t ModeNodeCtxOf<'t, K>,
    ) -> PatternCtxs<'t, K> {
        let part = self.clone().as_partition();
        part.offsets.ids().map(|id|
            (*id, ctx.as_pattern_context(id))
        ).collect()
    }

    /// bundle pattern range infos of each pattern
    /// or extract complete child for range
    fn partition_borders<
        't,
        C: PartitionBorderKey + From<ModePatternCtxOf<'t, K>>,
    >(
        self,
        ctx: &'t ModeNodeCtxOf<'t, K>,
    ) -> PartitionBorders<K, C> {
        let ctxs = self.pattern_ctxs(ctx);
        let (borders, perfect): (HashMap<_, _>, K::Perfect) = ctxs
            .into_iter()
            .map(|(_, pctx)| {
                let (perfect, borders) = {
                    let pctx = pctx.as_pattern_trace_context();
                    let borders = self.info_borders(
                        &pctx
                    );
                    (
                        borders.perfect().then_some(pctx.loc.id),
                        borders,
                    )
                };
                ((C::from(pctx), borders), perfect)
            })
            .unzip();
        PartitionBorders {
            borders,
            perfect,
        }
    }
    fn info_partition<'t>(
        self,
        ctx: &'t ModeNodeCtxOf<'t, K>,
    ) -> Result<PartitionInfo<K>, Child> {
        let borders = self.partition_borders(
            ctx,
        );
        PartitionInfo::from_partition_borders(
            borders,
        )
    }
}
impl<K: RangeRole, P: AsPartition<K>> VisitPartition<K> for P {
}