use std::{
    fmt::Debug,
    num::NonZeroUsize,
    ops::{
        Range,
        RangeFrom,
    },
};

use crate::{
    graph::vertex::pattern::{
        id::PatternId,
        pattern_range::PatternRangeIndex,
    },
    interval::{
        cache::{
            position_splits,
            range_splits,
            vertex::SplitVertexCache,
        },
        partition::{
            context::AsNodeTraceContext,
            info::range::{
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
            location::{
                PosSplitContext,
                ToVertexSplits,
                VertexSplits,
            },
        },
        PatternSplitPos,
    },
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
            .map(PosSplitContext::from)
            .nth(self.start)
            .unwrap();
        let ro = vertex
            .positions
            .iter()
            .map(PosSplitContext::from)
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
            .map(PosSplitContext::from)
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
            .map(PosSplitContext::from)
            .nth(self.start)
            .unwrap();
        lo.to_vertex_splits()
    }
}
pub trait RangeOffsets<R: RangeRole>: Debug + Clone + Copy {
    fn as_splits<C: AsNodeTraceContext>(
        &self,
        ctx: C,
    ) -> R::Splits;
}

impl<M: InVisitMode> RangeOffsets<In<M>> for (NonZeroUsize, NonZeroUsize) {
    fn as_splits<C: AsNodeTraceContext>(
        &self,
        ctx: C,
    ) -> <In<M> as RangeRole>::Splits {
        range_splits(ctx.as_trace_context().patterns.iter(), *self)
    }
}

impl<M: PreVisitMode> RangeOffsets<Pre<M>> for NonZeroUsize {
    fn as_splits<C: AsNodeTraceContext>(
        &self,
        ctx: C,
    ) -> <Pre<M> as RangeRole>::Splits {
        position_splits(ctx.as_trace_context().patterns.iter(), *self)
    }
}

impl<M: PostVisitMode> RangeOffsets<Post<M>> for NonZeroUsize {
    fn as_splits<C: AsNodeTraceContext>(
        &self,
        ctx: C,
    ) -> <Post<M> as RangeRole>::Splits {
        position_splits(ctx.as_trace_context().patterns.iter(), *self)
    }
}

pub trait PatternSplits: Debug + Clone {
    type Pos;
    type Offsets;
    fn get(
        &self,
        pid: &PatternId,
    ) -> Option<Self::Pos>;
    fn ids<'a>(&'a self) -> Box<dyn Iterator<Item = &'a PatternId> + 'a>;
    fn offsets(&self) -> Self::Offsets;
}

//pub trait PatternSplitsRef<'a>: PatternSplits + Copy + 'a {
//    type Ref<'t>: Copy where Self: 't;
//    fn as_ref<'t>(&'t self) -> Self::Ref<'t> where Self: 't;
//}
//impl<'a> PatternSplits for PosSplitRef<'a> {
//    type Pos = PatternSplitPos;
//    type Offsets = usize;
//    fn get(&self, pid: &PatternId) -> Option<Self::Pos> {
//        self.splits.get(pid).cloned()
//    }
//    fn offsets(&self) -> Self::Offsets {
//        self.offset.get()
//    }
//}
//impl<'a> PatternSplitsRef<'a> for PosSplitRef<'a> {
//    type Ref<'t> = Self where Self: 't;
//    fn as_ref<'t>(&'t self) -> Self::Ref<'t> where Self: 't {
//        *self
//    }
//}
impl PatternSplits for VertexSplits {
    type Pos = PatternSplitPos;
    type Offsets = usize;
    fn get(
        &self,
        pid: &PatternId,
    ) -> Option<Self::Pos> {
        self.splits.get(pid).cloned()
    }
    fn ids<'a>(&'a self) -> Box<dyn Iterator<Item = &'a PatternId> + 'a> {
        Box::new(self.splits.keys())
    }
    fn offsets(&self) -> Self::Offsets {
        self.pos.get()
    }
}

impl PatternSplits for &VertexSplits {
    type Pos = PatternSplitPos;
    type Offsets = usize;
    fn get(
        &self,
        pid: &PatternId,
    ) -> Option<Self::Pos> {
        self.splits.get(pid).cloned()
    }
    fn ids<'a>(&'a self) -> Box<dyn Iterator<Item = &'a PatternId> + 'a> {
        Box::new(self.splits.keys())
    }
    fn offsets(&self) -> Self::Offsets {
        self.pos.get()
    }
}

//impl<'a> PatternSplitsRef<'a> for &'a VertexSplits {
//    type Ref<'t> = Self where Self: 't;
//    fn as_ref<'t>(&'t self) -> Self::Ref<'t> where Self: 't {
//        *self
//    }
//}
impl<A: PatternSplits, B: PatternSplits> PatternSplits for (A, B) {
    type Pos = (A::Pos, B::Pos);
    type Offsets = (A::Offsets, B::Offsets);
    fn get(
        &self,
        pid: &PatternId,
    ) -> Option<Self::Pos> {
        self.0.get(pid).map(|a| {
            let b = self.1.get(pid).unwrap();
            (a, b)
        })
    }
    fn ids<'a>(&'a self) -> Box<dyn Iterator<Item = &'a PatternId> + 'a> {
        self.0.ids()
    }
    fn offsets(&self) -> Self::Offsets {
        (self.0.offsets(), self.1.offsets())
    }
}
//impl<
//    'a,
//    A: PatternSplitsRef<'a, Ref<'a> = PosSplitRef<'a>> + 'a,
//    B: PatternSplitsRef<'a, Ref<'a> = PosSplitRef<'a>> + 'a,
//> PatternSplitsRef<'a> for (A, B) {
//    type Ref<'t> = (PosSplitRef<'t>, PosSplitRef<'t>) where Self: 't;
//    fn as_ref<'t>(&'t self) -> Self::Ref<'t> where Self: 't {
//        (
//            self.0.as_ref(),
//            self.1.as_ref(),
//        )
//    }
//}
