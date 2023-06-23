use crate::*;

pub mod range;
pub use range::*;
pub mod border;
pub use border::*;

#[derive(Debug)]
pub struct PatternRangeInfo<K: RangeRole>
    where K::Mode: ModeChildren::<K>
{
    pub pattern_id: PatternId,
    pub info: RangeInfo<K>,
    pub perfect: <K::Perfect as BorderPerfect>::Boolean,
}
pub type PatternCtxOf<'a, K> = <ModeCtxOf<'a, K> as AsTraceContext<'a>>::PatternCtx<'a>;
pub type ModeCtxOf<'a, K> = <<K as RangeRole>::Mode as ModeContext<'a>>::Result;
pub type ModePatternCtxOf<'a, K> = <<K as RangeRole>::Mode as ModeContext<'a>>::PatternResult;

pub trait ModeContext<'a> {
    type Result: AsTraceContext<'a, PatternCtx<'a> = Self::PatternResult>;
    type PatternResult: AsPatternTraceContext<'a>;
}
impl<'a> ModeContext<'a> for Trace {
    type Result = TraceContext<'a>;
    type PatternResult = PatternTraceContext<'a>;
}
impl<'a> ModeContext<'a> for Join {
    type Result = JoinContext<'a>;
    type PatternResult = PatternJoinContext<'a>;
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
        //let offsets = part.offsets.offsets();

        K::Borders::make_borders(pctx.pattern, &splits)
            .join_inner_info(&ctx)
    }
}
pub trait JoinPartition<K: RangeRole<Mode = Join>>: VisitPartition<K> {
    fn join_partition<'a>(
        self,
        ctx: &mut JoinContext<'a>,
    ) -> Result<JoinedPartition<K>, Child>
    {
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

pub fn to_non_zero_range(
    l: usize,
    r: usize,
) -> (NonZeroUsize, NonZeroUsize) {
    (
        NonZeroUsize::new(l).unwrap(),
        NonZeroUsize::new(r).unwrap(),
    )
}
#[cfg(tests)]
mod tests {
    fn first_partition() {

    }
    fn inner_partition() {
        let cache = SplitCache {
            entries: HashMap::from([]),
            leaves: vec![],
        };
        let patterns = vec![

        ];
        let (lo, ro) = to_non_zero_range(1, 3);
        let (ls, rs) = range_splits(&patterns, (lo, ro));
        let (l, r) = ((&lo, ls), (&ro, rs)); 
        let bundle = (l, r).info_bundle();
    }
    fn last_partition() {

    }
}