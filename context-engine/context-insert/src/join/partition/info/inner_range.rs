use crate::{
    interval::partition::info::range::{
        InnerRangeInfo,
        role::RangeRole,
        splits::RangeOffsets,
    },
    join::{
        context::{
            node::context::NodeJoinCtx,
            pattern::borders::JoinBorders,
        },
        partition::{
            Join,
            JoinPartition,
        },
    },
};
use context_trace::{
    graph::vertex::child::Child,
    trace::node::AsNodeTraceCtx,
};
use derive_more::derive::{
    Deref,
    DerefMut,
    From,
    Into,
};
use derive_new::new;

#[derive(Debug, Clone, Deref, DerefMut, Into, From, new)]
pub struct JoinInnerRangeInfo<R: RangeRole<Mode = Join>>(InnerRangeInfo<R>)
where
    R::Borders: JoinBorders<R>;

impl<R: RangeRole<Mode = Join>> JoinInnerRangeInfo<R>
where
    R::Borders: JoinBorders<R>,
{
    pub fn index_pattern_inner<'a: 'b, 'b: 'c, 'c>(
        &self,
        ctx: &'c mut NodeJoinCtx<'a>,
    ) -> Child
    where
        Self: 'a,
    {
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
