use crate::*;

pub mod range;
pub use range::*;
pub mod border;
pub use border::*;

pub trait JoinPartition<K: RangeRole<Mode = Join>>: VisitPartition<K>
    where K::Borders: JoinBorders<K>
{
    fn join_partition<'a>(
        self,
        ctx: &mut NodeJoinContext<'a>,
    ) -> Result<JoinedPartition<K>, Child> {
        match self.info_partition(ctx) {
            Ok(info) => Ok(JoinedPartition::from_partition_info(info, ctx)),
            Err(c) => Err(c),
        }
    }
}
impl<K: RangeRole<Mode = Join>, P: VisitPartition<K>> JoinPartition<K> for P
    where K::Borders: JoinBorders<K>
{
}
pub trait VisitPartition<K: RangeRole>: Sized + Clone {
    fn info_borders<'t>(
        &self,
        ctx: PatternTraceContext,
    ) -> K::Borders;

    fn pattern_ctxs<'t>(
        &self,
        ctx: &'t ModeNodeCtxOf<'t, K>,
    ) -> HashMap<PatternId, ModePatternCtxOf<'t, K>>;

    /// bundle pattern range infos of each pattern
    /// or extract complete child for range
    fn info_partition<'t>(
        self,
        ctx: &'t ModeNodeCtxOf<'t, K>,
    ) -> Result<PartitionInfo<K>, Child> {
        let ctxs = self.pattern_ctxs(ctx);
        let (borders, perfect): (Vec<_>, K::Perfect) = ctxs
            .into_iter()
            .map(|(_, pctx)| {
                let (perfect, borders) = {
                    let pctx = pctx.as_pattern_trace_context();
                    let borders = self.info_borders(
                            pctx
                        );
                    (
                        borders.perfect().then_some(pctx.loc.id),
                        borders,
                    )
                };
                ((pctx, borders), perfect)
            })
            .unzip();
        let patterns: Result<_, _> = borders
            .into_iter()
            .sorted_by_key(|(_, borders)| !borders.perfect().all_perfect())
            .map(|(pctx, borders)|
                RangeInfoOf::<K>::info_pattern_range(
                    borders,
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
impl<K: RangeRole, P: AsPartition<K>> VisitPartition<K> for P {
    fn info_borders<'t>(
        &self,
        ctx: PatternTraceContext,
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
    ) -> HashMap<PatternId, ModePatternCtxOf<'t, K>> {
        let part = self.clone().as_partition();
        part.offsets.ids().map(|id|
            (*id, ctx.as_pattern_context(id))
        ).collect()
    }
}
impl<K: RangeRole> Into<(PatternId, RangeInfoOf<K>)> for PatternRangeInfo<K> {
    fn into(self) -> (PatternId, RangeInfoOf<K>) {
        (
            self.pattern_id,
            self.info,
        )
    }
}

#[derive(Debug, Default)]
pub struct PartitionInfo<K: RangeRole> {
    pub patterns: HashMap<PatternId, RangeInfoOf<K>>,
    pub perfect: K::Perfect,
}

impl<'a, K: RangeRole<Mode = Join>> PartitionInfo<K> 
    where K::Borders: JoinBorders<K>
{
    pub fn to_joined_patterns(
        self,
        ctx: &mut NodeJoinContext<'a>,
    ) -> JoinedPatterns<K>
    {
        JoinedPatterns::from_partition_info(self, ctx)
    }
    pub fn to_joined_partition(
        self,
        ctx: &mut NodeJoinContext<'a>,
    ) -> JoinedPartition<K> {
        JoinedPartition::from_partition_info(self, ctx)
    }
}
//pub(crate) trait TracePartition<'a, K: RangeRole>: VisitPartition<'a, K, NodeTraceContext<'a>> {
//}
//impl<'a, K: RangeRole, P: VisitPartition<'a, K, NodeTraceContext<'a>>> TracePartition<'a, K> for P {
//}
