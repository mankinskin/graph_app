use std::{
    fmt::Debug,
    num::NonZeroUsize,
    ops::{
        Range,
        RangeFrom,
    },
};

use crate::{
    interval::partition::info::range::{
        mode::{
            InVisitMode,
            PostVisitMode,
            PreVisitMode,
        },
        role::{
            In,
            Post,
            Pre,
            RangeRole,
        },
    },
    split::{
        cache::vertex::SplitVertexCache,
        position_splits,
        range_splits,
        vertex::{
            PosSplitCtx,
            ToVertexSplits,
        },
    },
};
use context_trace::{
    graph::vertex::pattern::pattern_range::PatternRangeIndex,
    trace::node::AsNodeTraceCtx,
};

pub trait OffsetIndexRange<R: RangeRole>: PatternRangeIndex {
    fn get_splits(
        &self,
        vertex: &SplitVertexCache,
    ) -> R::Splits;
}

impl<M: InVisitMode> OffsetIndexRange<In<M>> for Range<usize> {
    fn get_splits(
        &self,
        vertex: &SplitVertexCache,
    ) -> <In<M> as RangeRole>::Splits {
        let lo = vertex
            .positions
            .iter()
            .map(PosSplitCtx::from)
            .nth(self.start)
            .unwrap();
        let ro = vertex
            .positions
            .iter()
            .map(PosSplitCtx::from)
            .nth(self.end)
            .unwrap();
        (lo.to_vertex_splits(), ro.to_vertex_splits())
    }
}

impl<M: PreVisitMode> OffsetIndexRange<Pre<M>> for Range<usize> {
    fn get_splits(
        &self,
        vertex: &SplitVertexCache,
    ) -> <Pre<M> as RangeRole>::Splits {
        let ro = vertex
            .positions
            .iter()
            .map(PosSplitCtx::from)
            .nth(self.end)
            .unwrap();
        ro.to_vertex_splits()
    }
}

impl<M: PostVisitMode> OffsetIndexRange<Post<M>> for RangeFrom<usize> {
    fn get_splits(
        &self,
        vertex: &SplitVertexCache,
    ) -> <Post<M> as RangeRole>::Splits {
        let lo = vertex
            .positions
            .iter()
            .map(PosSplitCtx::from)
            .nth(self.start)
            .unwrap();
        lo.to_vertex_splits()
    }
}
pub trait RangeOffsets<R: RangeRole>: Debug + Clone + Copy {
    fn as_splits<C: AsNodeTraceCtx>(
        &self,
        ctx: C,
    ) -> R::Splits;
}

impl<M: InVisitMode> RangeOffsets<In<M>> for (NonZeroUsize, NonZeroUsize) {
    fn as_splits<C: AsNodeTraceCtx>(
        &self,
        ctx: C,
    ) -> <In<M> as RangeRole>::Splits {
        range_splits(ctx.as_trace_context().patterns.iter(), *self)
    }
}

impl<M: PreVisitMode> RangeOffsets<Pre<M>> for NonZeroUsize {
    fn as_splits<C: AsNodeTraceCtx>(
        &self,
        ctx: C,
    ) -> <Pre<M> as RangeRole>::Splits {
        position_splits(ctx.as_trace_context().patterns.iter(), *self)
    }
}

impl<M: PostVisitMode> RangeOffsets<Post<M>> for NonZeroUsize {
    fn as_splits<C: AsNodeTraceCtx>(
        &self,
        ctx: C,
    ) -> <Post<M> as RangeRole>::Splits {
        position_splits(ctx.as_trace_context().patterns.iter(), *self)
    }
}
