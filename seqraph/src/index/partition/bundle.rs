use crate::*;

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
impl<'p, K: RangeRole> PartitionInfo<K> {
    pub fn join_patterns(
        self,
        ctx: &mut JoinContext<'p>,
    ) -> JoinedPatterns<K> {
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
        ctx: &mut JoinContext<'p>,
    ) -> JoinedPartition<K> {
        // collect infos about partition in each pattern
        self.join_patterns(ctx).join(ctx)
    }
}