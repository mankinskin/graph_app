use crate::*;

#[derive(Debug)]
pub struct PatternRangeInfo<K: RangeRole> {
    pub pattern_id: PatternId,
    pub info: RangeInfo<K>,
    pub perfect: <K::Perfect as BorderPerfect>::Boolean,
}
pub(crate) trait JoinPartition<'a, K: RangeRole>: 'a + Sized {
    fn pattern_range_info(
        &self,
        pid: PatternId,
        pattern: &Pattern,
        //cache: &SplitCache,
        index: Child,
    ) -> Result<PatternRangeInfo<K>, Child>;

    /// bundle pattern range infos of each pattern
    fn info_bundle<'p: 'a, C: AsBundlingContext<'p>>(
        self,
        ctx: &C,
    ) -> Result<PartitionInfo<K>, Child> {
        let bctx = ctx.as_bundling_context();
        bctx.patterns.iter().map(|(pid, pattern)| {
            self.pattern_range_info(
                *pid,
                pattern,
                //bctx.cache,
                bctx.index,
            )
        })
        .collect()
    }
    fn join_partition<'p: 'a>(
        self,
        ctx: &mut JoinContext<'p>,
    ) -> Result<JoinedPartition<K>, Child> {
        match self.info_bundle(ctx) {
            Ok(bundle) => Ok(bundle.join(ctx)),
            Err(part) => Err(part),
        }
    }
}
impl<'p, K: RangeRole, P: AsPartition<'p, K>> JoinPartition<'p, K> for P {
    fn pattern_range_info(
        &self,
        pid: PatternId,
        pattern: &Pattern,
        //cache: &SplitCache,
        index: Child,
    ) -> Result<PatternRangeInfo<K>, Child> {
        let part = self.as_partition();
        // todo detect if prev offset is in same index (to use inner partition as result)
        let splits = part.offsets.get(&pid).unwrap();
        let offsets = part.offsets.offsets();

        K::Borders::border_info(pattern, splits)
            .join_inner_info(pid, pattern)
    }
}
//impl<'p, K: RangeRole + 'p> JoinPartition<'p> for PartitionRef<'p, K> {
//    type Range = K;
//    fn pattern_range_info(
//        &self,
//        pid: PatternId,
//        pattern: &Pattern,
//        //cache: &SplitCache,
//        index: Child,
//    ) -> Result<PatternRangeInfo<Self::Range>, Child> {
//    }
//}
//impl<'p, K: RangeRole + 'p> JoinPartition<'p, K> for Partition<K> {
//    fn pattern_range_info(
//        &self,
//        pid: PatternId,
//        pattern: &Pattern,
//        //cache: &SplitCache,
//        index: Child,
//    ) -> Result<PatternRangeInfo<K>, Child> {
//        // todo detect if prev offset is in same index (to use inner partition as result)
//        let splits = self.offsets.get(&pid).unwrap();
//        let urange = self.offsets.offsets();
//
//        K::Borders::border_info(pattern, splits)
//            .join_inner_info(pid, pattern)
//    }
//}