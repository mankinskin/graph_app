use crate::join::{
    context::pattern::borders::JoinBorders,
    joined::{
        partition::JoinedPartition,
        patterns::JoinedPatterns,
    },
};
use derive_more::derive::{
    Deref,
    DerefMut,
    From,
    Into,
};
use derive_new::new;
use crate::interval::partition::info::{
    range::role::{
        ModeNodeCtxOf,
        RangeRole,
    },
    PartitionInfo,
};

use super::Join;
pub mod inner_range;
pub mod pattern_info;

#[derive(Debug, Deref, DerefMut, Into, From, new)]
pub struct JoinPartitionInfo<R: RangeRole<Mode = Join>>(PartitionInfo<R>)
where
    R::Borders: JoinBorders<R>;

impl<'a: 'b, 'b: 'c, 'c, R: RangeRole<Mode = Join>> JoinPartitionInfo<R>
where
    R::Borders: JoinBorders<R>,
    Self: 'a,
{
    pub fn to_joined_patterns(
        self,
        ctx: &'c mut ModeNodeCtxOf<'a, 'b, R>,
    ) -> JoinedPatterns<R> {
        JoinedPatterns::from_partition_info(self, ctx)
    }
    pub fn to_joined_partition(
        self,
        ctx: &'c mut ModeNodeCtxOf<'a, 'b, R>,
    ) -> JoinedPartition<R> {
        JoinedPartition::from_partition_info(self, ctx)
    }
}
