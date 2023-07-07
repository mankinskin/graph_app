use crate::*;

pub mod range;
pub use range::*;
pub mod border;
pub use border::*;

#[derive(Debug, Default)]
pub struct PartitionInfo<K: RangeRole> {
    pub patterns: HashMap<PatternId, RangeInfo<K>>,
    pub perfect: K::Perfect,
}

impl<K: RangeRole> FromIterator<PatternRangeInfo<K>> for PartitionInfo<K> {
    fn from_iter<T: IntoIterator<Item = PatternRangeInfo<K>>>(iter: T) -> Self {
        let mut perf = K::Perfect::default();
        let patterns =
            iter.into_iter()
                .map(|PatternRangeInfo {
                    pattern_id,
                    info,
                    perfect,
                }| {
                    perf.fold_or(perfect.then_some(pattern_id));
                    (pattern_id, info)
                })
                .collect();
        PartitionInfo {
            patterns,
            perfect: perf,
            //delta,
        }
    }
}
impl<'a, K: RangeRole<Mode = Join>> PartitionInfo<K> {
    pub fn join_patterns(
        self,
        ctx: &mut JoinContext<'a>,
    ) -> JoinedPatterns<K>
    {
        let (delta, patterns) = self.patterns.into_iter()
            .map(|(pattern_id, info)|
                (
                    (pattern_id, info.delta),
                    info.join_pattern_inner(pattern_id, ctx),
                )
            )
            .unzip();
        JoinedPatterns {
            patterns,
            perfect: self.perfect,
            delta,
        }
    }
    pub fn join(
        self,
        ctx: &mut JoinContext<'a>,
    ) -> JoinedPartition<K>
    {
        // collect infos about partition in each pattern
        self.join_patterns(ctx).join(ctx)
    }
}
pub trait VisitPartition<K: RangeRole>: Sized + Clone {
    fn visit_pattern_range<'t>(
        self,
        ctx: ModePatternCtxOf<'t, K>,
    ) -> Result<PatternRangeInfo<K>, Child>;

    /// bundle pattern range infos of each pattern
    fn visit_partition<'t>(
        self,
        ctx: &'t ModeCtxOf<'t, K>,
    ) -> Result<PartitionInfo<K>, Child> {
        let tctx = ctx.as_trace_context();
        tctx.patterns.iter().map(|(&pid, pattern)|
            self.clone().visit_pattern_range(
                ctx.pattern_context(pid, pattern)
            )
        )
        .collect()
    }
}
impl<K: RangeRole, P: AsPartition<K>> VisitPartition<K> for P {
    fn visit_pattern_range<'t>(
        self,
        ctx: ModePatternCtxOf<'t, K>,
    ) -> Result<PatternRangeInfo<K>, Child> 
    {
        let part = self.as_partition();
        // todo detect if prev offset is in same index (to use inner partition as result)
        let pctx = ctx.as_pattern_trace_context();
        let splits = part.offsets.get(&pctx.pattern_id).unwrap();

        K::Borders::make_borders(pctx.pattern, &splits)
            .join_inner_info(&ctx)
    }
}
pub trait JoinPartition<K: RangeRole<Mode = Join>>: VisitPartition<K> {
    fn join_partition<'a>(
        self,
        ctx: &mut JoinContext<'a>,
    ) -> Result<JoinedPartition<K>, Child> {
        match self.visit_partition(ctx) {
            Ok(info) => Ok(info.join(ctx)),
            Err(c) => Err(c),
        }
    }
}
impl<'a, K: RangeRole<Mode = Join> + 'a, P: VisitPartition<K>> JoinPartition<K> for P {
}

//pub(crate) trait TracePartition<'a, K: RangeRole>: VisitPartition<'a, K, TraceContext<'a>> {
//}
//impl<'a, K: RangeRole, P: VisitPartition<'a, K, TraceContext<'a>>> TracePartition<'a, K> for P {
//}
