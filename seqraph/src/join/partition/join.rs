use crate::join::{
        context::node::context::NodeJoinContext,
        joined::partition::JoinedPartition,
    };
use hypercontext_api::{
    graph::vertex::child::Child,
    partition::info::{
        range::role::RangeRole,
        InfoPartition,
    },
};

use super::{
    borders::JoinBorders,
    Join, JoinPartitionInfo,
};

pub trait JoinPartition<R: RangeRole<Mode = Join>>: InfoPartition<R>
where
    R::Borders: JoinBorders<R>,
{
    fn join_partition<'a: 'b, 'b: 'c, 'c>(
        self,
        ctx: &'c mut NodeJoinContext<'a>,
    ) -> Result<JoinedPartition<R>, Child>
    where
        Self: 'c,
        R: 'a,
    {
        match self.info_partition(ctx) {
            Ok(info) => Ok(JoinPartitionInfo(info).to_joined_partition(ctx)),
            Err(c) => Err(c),
        }
    }
}

impl<R: RangeRole<Mode = Join>, P: InfoPartition<R>> JoinPartition<R> for P where
    R::Borders: JoinBorders<R>
{
}
