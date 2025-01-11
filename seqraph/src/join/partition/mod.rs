pub mod borders;
pub mod info;
pub mod join;

use borders::JoinBorders;
use derive_more::derive::{Deref, DerefMut, From, Into};
use info::JoinPatternInfo;

use crate::join::{
        context::node::context::NodeJoinContext,
        joined::{
            partition::JoinedPartition,
            patterns::JoinedPatterns,
        },
    };
use hypercontext_api::partition::info::{
        range::{
            mode::{InVisitMode, ModeChildren, ModeContext, ModeInfo, PostVisitMode, PreVisitMode},
            role::{
                ModeNodeCtxOf,
                RangeRole,
            },
        },
        PartitionInfo,
};

use super::context::pattern::PatternJoinContext;

#[derive(Debug, Clone, Copy)]
pub struct Join;

impl ModeContext for Join {
    type NodeContext<'a: 'b, 'b> = NodeJoinContext<'a>;
    type PatternResult<'a> = PatternJoinContext<'a>;
}

impl<R: RangeRole<Mode = Join>> ModeChildren<R> for Join {
    type Result = R::Children;
}

impl PreVisitMode for Join {}
impl PostVisitMode for Join {}
impl InVisitMode for Join {}

#[derive(Debug, Deref, DerefMut, Into, From)]
pub struct JoinPartitionInfo<R: RangeRole<Mode = Join>>(PartitionInfo<R>)
where
    R::Borders: JoinBorders<R>,
;

impl<'a: 'b, 'b: 'c, 'c, R: RangeRole<Mode = Join>> JoinPartitionInfo<R>
where
    R::Borders: JoinBorders<R>,
    Self: 'a,
{
    pub fn to_joined_patterns(
        self,
        ctx: &'c mut ModeNodeCtxOf<'a, 'b, R>,
    ) -> JoinedPatterns<R>
    {
        JoinedPatterns::from_partition_info(self, ctx)
    }
    pub fn to_joined_partition(
        self,
        ctx: &'c mut ModeNodeCtxOf<'a, 'b, R>,
    ) -> JoinedPartition<R>
    {
        JoinedPartition::from_partition_info(self, ctx)
    }
}

impl<R: RangeRole<Mode = Self>> ModeInfo<R> for Join
where
    R::Borders: JoinBorders<R>,
{
    type PatternInfo = JoinPatternInfo<R>;
}
