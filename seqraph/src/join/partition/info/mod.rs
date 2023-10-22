use crate::*;

pub mod range;
pub use range::*;
pub mod border;
pub use border::*;

pub trait JoinPartition<K: RangeRole<Mode = Join>>: VisitPartition<K> {
    fn join_partition<'a>(
        self,
        ctx: &mut NodeJoinContext<'a>,
    ) -> Result<JoinedPartition<K>, Child> {
        match self.info_partition(ctx) {
            Ok(info) => Ok(info.join_info(ctx)),
            Err(c) => Err(c),
        }
    }
}
impl<'a, K: RangeRole<Mode = Join> + 'a, P: VisitPartition<K>> JoinPartition<K> for P {
}
pub trait VisitPartition<K: RangeRole>: Sized + Clone {
    //fn info_pattern_range<'t>(
    //    self,
    //    ctx: ModePatternCtxOf<'t, K>,
    //) -> Result<PatternRangeInfo<K>, Child>;
    fn info_borders<'t>(
        self,
        ctx: PatternTraceContext,
    ) -> K::Borders<'t>;

    fn pattern_ctxs<'t>(
        &self,
        ctx: &'t ModeNodeCtxOf<'t, K>,
    ) -> Vec<ModePatternCtxOf<'t, K>>;

    /// bundle pattern range infos of each pattern
    /// or extract complete child for range
    fn info_partition<'t>(
        self,
        ctx: &'t ModeNodeCtxOf<'t, K>,
    ) -> Result<PartitionInfo<K>, Child> {
        let ctxs = self.pattern_ctxs(ctx);
        let (borders, perfect): (Vec<_>, K::Perfect) = ctxs
            .iter()
            .map(|pctx| {
                let pctx = pctx.as_pattern_trace_context();
                let borders = self.clone().info_borders(
                    pctx
                );
                let perfect = borders.perfect().then_some(pctx.loc.id);
                (borders, perfect)
            })
            .unzip();
        let patterns: Result<_, _> = ctxs.into_iter().zip(borders)
            .map(|(pctx, borders)|
                borders.info_pattern_range(
                    &pctx
                ).map(Into::into)
            )
            .collect();
        patterns.map(|infos|
            PartitionInfo {
                patterns: infos,
                perfect,
            }
        )
    }
}
impl<K: RangeRole> Into<(PatternId, RangeInfo<K>)> for PatternRangeInfo<K> {
    fn into(self) -> (PatternId, RangeInfo<K>) {
        (
            self.pattern_id,
            self.info,
        )
    }
}
impl<K: RangeRole, P: AsPartition<K>> VisitPartition<K> for P {
    fn info_borders<'t>(
        self,
        ctx: PatternTraceContext,
    ) -> K::Borders<'t> {
        let part = self.as_partition();
        // todo detect if prev offset is in same index (to use inner partition as result)
        let pctx = ctx.as_pattern_trace_context();
        let splits = part.offsets.get(&pctx.loc.id).unwrap();

        K::Borders::info_border(pctx.pattern, &splits)
    }
    fn pattern_ctxs<'t>(
        &self,
        ctx: &'t ModeNodeCtxOf<'t, K>,
    ) -> Vec<ModePatternCtxOf<'t, K>> {
        let part = self.clone().as_partition();
        part.offsets.ids().map(|id|
            ctx.as_pattern_context(id)
        ).collect_vec()
    }
}

#[derive(Debug, Default)]
pub struct PartitionInfo<K: RangeRole> {
    pub patterns: HashMap<PatternId, RangeInfo<K>>,
    pub perfect: K::Perfect,
}

impl<'a, K: RangeRole<Mode = Join>> PartitionInfo<K> {
    pub fn join_patterns(
        self,
        ctx: &mut NodeJoinContext<'a>,
    ) -> JoinedPatterns<K>
    {
        // assert: no complete perfect child
        let perfect = self.perfect;
        // todo: index inner ranges and get child splits
        //
        // index inner range
        // cases:
        // - (child, inner, child)
        // - (child, inner)
        // - (inner, child),
        // - (child, child),
        // - child: not possible, handled earlier
        let (delta, patterns) = self.patterns.into_iter()
            .map(|(pid, info)| {
                let delta = info.delta;
                let pattern = info.joined_pattern(ctx, &pid);
                (
                    (pid, delta),
                    pattern,
                )
            })
            .unzip();

        JoinedPatterns {
            patterns,
            perfect,
            delta,
        }
    }
    pub fn join_info(
        self,
        ctx: &mut NodeJoinContext<'a>,
    ) -> JoinedPartition<K>
    {
        // collect infos about partition in each pattern
        self.join_patterns(ctx).insert_patterns(ctx)
    }
}
//pub(crate) trait TracePartition<'a, K: RangeRole>: VisitPartition<'a, K, NodeTraceContext<'a>> {
//}
//impl<'a, K: RangeRole, P: VisitPartition<'a, K, NodeTraceContext<'a>>> TracePartition<'a, K> for P {
//}
