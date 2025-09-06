use border::{
    PartitionBorder,
    perfect::BoolPerfect,
};
use borders::PartitionBorders;

use range::{
    mode::PatternInfoOf,
    role::{
        ModePatternCtxOf,
        RangeRole,
    },
};
use std::hash::Hash;

use crate::{
    interval::partition::{
        ToPartition,
        info::{
            border::visit::VisitBorders,
            range::role::ModeNodeCtxOf,
        },
    },
    split::pattern::PatternSplits,
};
use context_trace::{
    HashMap,
    graph::vertex::{
        child::Child,
        pattern::id::PatternId,
    },
    trace::pattern::{
        GetPatternCtx,
        HasPatternTraceCtx,
        PatternTraceCtx,
    },
};

pub mod border;
pub mod borders;
pub mod range;

#[derive(Debug, Default)]
pub struct PartitionInfo<R: RangeRole> {
    pub patterns: HashMap<PatternId, PatternInfoOf<R>>,
    pub perfect: R::Perfect,
}

pub type PatternCtxs<'t, R> = HashMap<PatternId, ModePatternCtxOf<'t, R>>;

pub trait PartitionBorderKey: Hash + Eq {}

impl<T: Hash + Eq> PartitionBorderKey for T {}
pub trait InfoPartition<R: RangeRole>: Sized + Clone + ToPartition<R> {
    fn info_borders(
        &self,
        ctx: &PatternTraceCtx,
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
    ) -> PatternCtxs<'b, R> {
        let part = self.clone().to_partition();
        part.offsets
            .ids()
            .map(|id| (*id, ctx.get_pattern_context(id)))
            .collect()
    }

    /// bundle pattern range infos of each pattern
    /// or extract complete child for range
    fn partition_borders<
        'a: 'b,
        'b,
        C: PartitionBorderKey + From<ModePatternCtxOf<'b, R>>,
    >(
        &'b self,
        ctx: &'b ModeNodeCtxOf<'a, 'b, R>,
    ) -> PartitionBorders<R, C> {
        let ctxs = self.pattern_ctxs(ctx);
        let (perfect, borders): (R::Perfect, HashMap<_, _>) = ctxs
            .into_values()
            .map(|pctx| {
                let (perfect, borders) = {
                    let pctx = pctx.pattern_trace_context();
                    let borders = self.info_borders(&pctx);
                    (borders.perfect().then_some(pctx.loc.id), borders)
                };
                (perfect, (C::from(pctx), borders))
            })
            .unzip();
        PartitionBorders { borders, perfect }
    }
    fn info_partition<'a: 'b, 'b>(
        &'b self,
        ctx: &'b ModeNodeCtxOf<'a, 'b, R>,
    ) -> Result<PartitionInfo<R>, Child> {
        self.partition_borders(ctx).into_partition_info()
    }
}

impl<R: RangeRole, P: ToPartition<R>> InfoPartition<R> for P {}
