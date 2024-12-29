use crate::{
    graph::vertex::child::Child, join::{
        context::node::context::NodeJoinContext,
        partition::{borders::JoinBorders, Join},
    }, partition::{
        context::AsNodeTraceContext, info::range::{
            role::RangeRole,
            splits::RangeOffsets, InnerRangeInfo,
        },
    }
};

impl<R: RangeRole<Mode = Join>> InnerRangeInfo<R>
where
    R::Borders: JoinBorders<R>,
{
    pub fn index_pattern_inner(
        &self,
        ctx: &mut NodeJoinContext,
    ) -> Child {
        match self
            .offsets
            .as_splits(ctx.as_trace_context())
            .join_partition(ctx)
        {
            Ok(inner) => inner.index,
            Err(p) => p,
        }
    }
}
