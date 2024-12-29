use std::hash::Hash;

use crate::{
    partition::{
        info::{
            border::{
                perfect::BoolPerfect, visit::VisitBorders, PartitionBorder
            }, range::{
                role::{
                    ModeNodeCtxOf,
                    ModePatternCtxOf,
                    RangeRole,
                },
                splits::PatternSplits,
            }, PartitionInfo
        }, pattern::{AsPatternContext, AsPatternTraceContext, PatternTraceContext}, ToPartition
    }, HashMap
};
use crate::graph::vertex::{
    child::Child,
    pattern::id::PatternId,
};

pub struct PartitionBorders<R: RangeRole, C: PartitionBorderKey = PatternId> {
    pub borders: HashMap<C, R::Borders>,
    pub perfect: R::Perfect,
}

pub type PatternCtxs<'t, R> = HashMap<PatternId, ModePatternCtxOf<'t, R>>;

pub trait PartitionBorderKey: Hash + Eq {}

impl<T: Hash + Eq> PartitionBorderKey for T {}

pub trait VisitPartition<R: RangeRole>: Sized + Clone + ToPartition<R> {
    fn info_borders(
        &self,
        ctx: &PatternTraceContext,
    ) -> R::Borders {
        let part = self.clone().to_partition();
        // todo detect if prev offset is in same index (to use inner partition as result)
        let pctx = ctx.as_pattern_trace_context();
        let splits = part.offsets.get(&pctx.loc.id).unwrap();

        R::Borders::info_border(pctx.pattern, &splits)
    }

    fn pattern_ctxs<'t, 'p: 't>(
        &self,
        ctx: &'p ModeNodeCtxOf<'t, R>,
    ) -> PatternCtxs<'t, R> {
        let part = self.clone().to_partition();
        part.offsets
            .ids()
            .map(|id| (*id, ctx.as_pattern_context(id)))
            .collect()
    }

    /// bundle pattern range infos of each pattern
    /// or extract complete child for range
    fn partition_borders<'t, 'p: 't, C: PartitionBorderKey + From<ModePatternCtxOf<'t, R>>>(
        self,
        ctx: &'p ModeNodeCtxOf<'t, R>,
    ) -> PartitionBorders<R, C>
    {
        let ctxs = self.pattern_ctxs(ctx);
        let (borders, perfect): (HashMap<_, _>, R::Perfect) = ctxs
            .into_values()
            .map(|pctx| {
                let (perfect, borders) = {
                    let pctx = pctx.as_pattern_trace_context();
                    let borders = self.info_borders(&pctx);
                    (borders.perfect().then_some(pctx.loc.id), borders)
                };
                ((C::from(pctx), borders), perfect)
            })
            .unzip();
        PartitionBorders { borders, perfect }
    }
    fn info_partition<'t, 'p: 't>(
        self,
        ctx: &'p ModeNodeCtxOf<'t, R>,
    ) -> Result<PartitionInfo<R>, Child> {
        let borders = self.partition_borders(ctx);
        PartitionInfo::from_partition_borders(borders)
    }
}

impl<R: RangeRole, P: ToPartition<R>> VisitPartition<R> for P {}
