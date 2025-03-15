use crate::{
    graph::vertex::child::Child,
    interval::partition::info::range::{
        role::RangeRole,
        splits::RangeOffsets,
        InnerRangeInfo,
    },
    join::{
        context::{
            node::context::NodeJoinContext,
            pattern::borders::JoinBorders,
        },
        partition::{
            Join,
            JoinPartition,
        },
    },
    traversal::trace::context::node::AsNodeTraceContext,
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
        ctx: &'c mut NodeJoinContext<'a>,
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
