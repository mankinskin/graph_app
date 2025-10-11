use itertools::Itertools;

use context_trace::*;

use crate::interval::partition::info::{
    PartitionBorderKey,
    PartitionInfo,
    border::{
        PartitionBorder,
        perfect::BoolPerfect,
    },
    range::{
        ModeRangeInfo,
        mode::PatternInfoOf,
        role::{
            ModePatternCtxOf,
            RangeRole,
        },
    },
};
pub struct PartitionBorders<R: RangeRole, C: PartitionBorderKey = PatternId> {
    pub borders: HashMap<C, R::Borders>,
    pub perfect: R::Perfect,
}

impl<R: RangeRole> PartitionBorders<R, ModePatternCtxOf<'_, R>> {
    pub fn into_partition_info(self) -> Result<PartitionInfo<R>, Child> {
        let perfect = self.perfect;
        let patterns: Result<_, _> = self
            .borders
            .into_iter()
            .sorted_by_key(|(_, borders)| !borders.perfect().all_perfect())
            .map(|(pctx, borders)| {
                PatternInfoOf::<R>::info_pattern_range(borders, &pctx)
                    .map(Into::into)
            })
            .collect();
        patterns.map(|infos| PartitionInfo {
            patterns: infos,
            perfect,
        })
    }
}
