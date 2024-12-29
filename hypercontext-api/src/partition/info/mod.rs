use border::{
    perfect::BoolPerfect,
    PartitionBorder,
};
use itertools::Itertools;

use range::{
    mode::RangeInfoOf,
    role::{
        ModePatternCtxOf,
        RangeRole,
    },
    ModeRangeInfo,
};
use visit::PartitionBorders;

use crate::{
    graph::vertex::{
        child::Child,
        pattern::id::PatternId,
    },
    HashMap,
};

pub mod border;
pub mod range;
pub mod visit;

#[derive(Debug, Default)]
pub struct PartitionInfo<R: RangeRole> {
    pub patterns: HashMap<PatternId, RangeInfoOf<R>>,
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
                RangeInfoOf::<R>::info_pattern_range(borders, &pctx).map(Into::into)
            })
            .collect();
        patterns.map(|infos| PartitionInfo {
            patterns: infos,
            perfect,
        })
    }
}
