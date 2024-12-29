pub mod borders;
pub mod info;
pub mod join;


use borders::JoinBorders;
use derivative::Derivative;
use derive_more::derive::{
    Deref,
    DerefMut,
};
use info::JoinRangeInfo;

use crate::{
    join::{
        context::node::context::NodeJoinContext,
        joined::{
            JoinedPartition,
            JoinedPatterns,
        },
    },
    partition::info::{
        range::{
            mode::VisitMode,
            role::{
                InVisitMode,
                ModeChildren,
                ModeContext,
                PostVisitMode,
                PreVisitMode,
                RangeRole,
            },
        },
        PartitionInfo,
    },
};

use super::context::pattern::PatternJoinContext;

#[derive(Debug, Clone, Copy)]
pub struct Join;

impl<'a> ModeContext<'a> for Join {
    type NodeResult = NodeJoinContext<'a>;
    type PatternResult = PatternJoinContext<'a>;
}

impl<R: RangeRole<Mode = Join>> ModeChildren<R> for Join {
    type Result = R::Children;
}

impl PreVisitMode for Join {}
impl PostVisitMode for Join {}
impl InVisitMode for Join {}

impl<'a, R: RangeRole<Mode = Join>> PartitionInfo<R>
where
    R::Borders: JoinBorders<R>,
{
    pub fn to_joined_patterns(
        self,
        ctx: &mut NodeJoinContext<'a>,
    ) -> JoinedPatterns<R> {
        JoinedPatterns::from_partition_info(self, ctx)
    }
    pub fn to_joined_partition(
        self,
        ctx: &mut NodeJoinContext<'a>,
    ) -> JoinedPartition<R> {
        JoinedPartition::from_partition_info(self, ctx)
    }
}


impl<R: RangeRole<Mode = Self>> VisitMode<R> for Join
where
    R::Borders: JoinBorders<R>,
{
    type RangeInfo = JoinRangeInfo<R>;
}

