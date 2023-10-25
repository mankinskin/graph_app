use crate::*;

pub mod range;
pub use range::*;
pub mod border;
pub use border::*;
pub mod visit;
pub use visit::*;

#[derive(Debug, Default)]
pub struct PartitionInfo<K: RangeRole> {
    pub patterns: HashMap<PatternId, RangeInfoOf<K>>,
    pub perfect: K::Perfect,
}

impl<'a, K: RangeRole> PartitionInfo<K> {
    pub fn from_partition_borders<'t>(
        borders: PartitionBorders<K, ModePatternCtxOf<'t, K>>,
    ) -> Result<PartitionInfo<K>, Child> {
        let perfect = borders.perfect;
        let patterns: Result<_, _> = borders.borders
            .into_iter()
            .sorted_by_key(|(_, borders)| !borders.perfect().all_perfect())
            .map(|(pctx, borders)|
                RangeInfoOf::<K>::info_pattern_range(
                    borders,
                    &pctx,
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
impl<K: RangeRole> Into<(PatternId, RangeInfoOf<K>)> for PatternRangeInfo<K> {
    fn into(self) -> (PatternId, RangeInfoOf<K>) {
        (
            self.pattern_id,
            self.info,
        )
    }
}
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
