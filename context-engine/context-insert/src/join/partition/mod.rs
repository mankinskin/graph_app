pub mod info;

use crate::{
    interval::partition::info::{
        InfoPartition,
        range::{
            mode::{
                InVisitMode,
                ModeChildren,
                ModeCtx,
                ModeInfo,
                PostVisitMode,
                PreVisitMode,
            },
            role::RangeRole,
        },
    },
    join::context::node::context::NodeJoinCtx,
};
use context_trace::graph::vertex::child::Child;
use info::{
    JoinPartitionInfo,
    pattern_info::JoinPatternInfo,
};

use super::{
    context::pattern::{
        PatternJoinCtx,
        borders::JoinBorders,
    },
    joined::partition::JoinedPartition,
};

pub trait JoinPartition<R: RangeRole<Mode = Join>>: InfoPartition<R>
where
    R::Borders: JoinBorders<R>,
{
    fn join_partition<'a: 'b, 'b: 'c, 'c>(
        self,
        ctx: &'c mut NodeJoinCtx<'a>,
    ) -> Result<JoinedPartition<R>, Child>
    where
        Self: 'c,
        R: 'a,
    {
        match self.info_partition(ctx) {
            Ok(info) =>
                Ok(JoinPartitionInfo::new(info).to_joined_partition(ctx)),
            Err(c) => Err(c),
        }
    }
}

impl<R: RangeRole<Mode = Join>, P: InfoPartition<R>> JoinPartition<R> for P where
    R::Borders: JoinBorders<R>
{
}

#[derive(Debug, Clone, Copy)]
pub struct Join;

impl ModeCtx for Join {
    type NodeCtx<'a: 'b, 'b> = NodeJoinCtx<'a>;
    type PatternResult<'a> = PatternJoinCtx<'a>;
}

impl<R: RangeRole<Mode = Join>> ModeChildren<R> for Join {
    type Result = R::Children;
}

impl PreVisitMode for Join {}
impl PostVisitMode for Join {}
impl InVisitMode for Join {}

impl<R: RangeRole<Mode = Self>> ModeInfo<R> for Join
where
    R::Borders: JoinBorders<R>,
{
    type PatternInfo = JoinPatternInfo<R>;
}
