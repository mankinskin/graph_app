use crate::{graph::vertex::child::Child, join::{context::node::context::NodeJoinContext, joined::JoinedPartition}, partition::info::{range::role::RangeRole, visit::VisitPartition}};

use super::{borders::JoinBorders, Join};

pub trait JoinPartition<R: RangeRole<Mode = Join>>: VisitPartition<R>
where
    R::Borders: JoinBorders<R>,
{
    fn join_partition<'t>(
        self,
        ctx: &mut NodeJoinContext<'t>,
    ) -> Result<JoinedPartition<R>, Child> {
        match self.info_partition(ctx) {
            Ok(info) => Ok(JoinedPartition::from_partition_info(info, ctx)),
            Err(c) => Err(c),
        }
    }
}

impl<R: RangeRole<Mode=Join>, P: VisitPartition<R>> JoinPartition<R> for P where
    R::Borders: JoinBorders<R>
{
}
