use border::{
    perfect::BoolPerfect,
    PartitionBorder,
};
use itertools::Itertools;

use range::{
    mode::PatternInfoOf,
    role::{
        ModePatternCtxOf,
        RangeRole,
    },
    ModeRangeInfo,
};

use crate::{
    graph::vertex::{
        child::Child,
        pattern::id::PatternId,
    },
    HashMap,
};
use std::hash::Hash;

use crate::partition::{
    info::{
        border::visit::VisitBorders,
        range::{
            role::ModeNodeCtxOf,
            splits::PatternSplits,
        },
    }, pattern::{GetPatternContext, HasPatternTraceContext, PatternTraceContext}, ToPartition
};

pub mod border;
pub mod range;

#[derive(Debug, Default)]
pub struct PartitionInfo<R: RangeRole> {
    pub patterns: HashMap<PatternId, PatternInfoOf<R>>,
    pub perfect: R::Perfect,
}

impl<R: RangeRole> PartitionInfo<R> {
    pub fn from_partition_borders(
        borders: PartitionBorders<R, ModePatternCtxOf<R>>
    ) -> Result<PartitionInfo<R>, Child> {
        let perfect = borders.perfect;
        let patterns: Result<_, _> = borders
            .borders
            .into_iter()
            .sorted_by_key(|(_, borders)| !borders.perfect().all_perfect())
            .map(|(pctx, borders)| {
                PatternInfoOf::<R>::info_pattern_range(borders, &pctx).map(Into::into)
            })
            .collect();
        patterns.map(|infos| PartitionInfo {
            patterns: infos,
            perfect,
        })
    }
}

pub struct PartitionBorders<R: RangeRole, C: PartitionBorderKey = PatternId> {
    pub borders: HashMap<C, R::Borders>,
    pub perfect: R::Perfect,
}

pub type PatternCtxs<'t, R> = HashMap<PatternId, ModePatternCtxOf<'t, R>>;

pub trait PartitionBorderKey: Hash + Eq {}

impl<T: Hash + Eq> PartitionBorderKey for T {}

pub trait InfoPartition<R: RangeRole>: Sized + Clone + ToPartition<R> {
    fn info_borders(
        &self,
        ctx: &PatternTraceContext,
    ) -> R::Borders {
        let part = self.clone().to_partition();
        // todo detect if prev offset is in same index (to use inner partition as result)
        let pctx = ctx.pattern_trace_context();
        let splits = part.offsets.get(&pctx.loc.id).unwrap();

        R::Borders::info_border(pctx.pattern, &splits)
    }

    fn pattern_ctxs<'a: 'b, 'b>(
        &'b self,
        ctx: &'b ModeNodeCtxOf<'a, 'b, R>,
    ) -> PatternCtxs<'b, R>
    {
        let part = self.clone().to_partition();
        part.offsets
            .ids()
            .map(|id| (*id, ctx.get_pattern_context(id)))
            .collect()
    }

    /// bundle pattern range infos of each pattern
    /// or extract complete child for range
    fn partition_borders<'a: 'b, 'b, C: PartitionBorderKey + From<ModePatternCtxOf<'b, R>>>(
        &'b self,
        ctx: &'b ModeNodeCtxOf<'a, 'b, R>,
    ) -> PartitionBorders<R, C>
    {
        let ctxs = self.pattern_ctxs(ctx);
        let (borders, perfect): (HashMap<_, _>, R::Perfect) = ctxs
            .into_values()
            .map(|pctx| {
                let (perfect, borders) = {
                    let pctx = pctx.pattern_trace_context();
                    let borders = self.info_borders(&pctx);
                    (borders.perfect().then_some(pctx.loc.id), borders)
                };
                ((C::from(pctx), borders), perfect)
            })
            .unzip();
        PartitionBorders { borders, perfect }
    }
    fn info_partition<'a: 'b, 'b>(
        &'b self,
        ctx: &'b ModeNodeCtxOf<'a, 'b, R>,
    ) -> Result<PartitionInfo<R>, Child>
    {
        let borders = self.partition_borders(ctx);
        PartitionInfo::from_partition_borders(borders)
    }
}

impl<R: RangeRole, P: ToPartition<R>> InfoPartition<R> for P {}