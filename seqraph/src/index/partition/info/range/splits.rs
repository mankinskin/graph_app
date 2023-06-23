use crate::*;

pub trait RangeOffsets<K: RangeRole>: Debug + Clone + Copy {
    fn as_splits<'a, C: AsTraceContext<'a>>(&'a self, ctx: C) -> K::Splits;
}
impl<M: InVisitMode> RangeOffsets<In<M>> for (NonZeroUsize, NonZeroUsize) {
    fn as_splits<'a, C: AsTraceContext<'a>>(&'a self, ctx: C) -> <In<M> as RangeRole>::Splits {
        range_splits(ctx.as_trace_context().patterns.iter(), *self)
    }
}
impl<M: PreVisitMode> RangeOffsets<Pre<M>> for NonZeroUsize {
    fn as_splits<'a, C: AsTraceContext<'a>>(&'a self, ctx: C) -> <Pre<M> as RangeRole>::Splits {
        position_splits(ctx.as_trace_context().patterns.iter(), *self)
    }
}
impl<M: PostVisitMode> RangeOffsets<Post<M>> for NonZeroUsize {
    fn as_splits<'a, C: AsTraceContext<'a>>(&'a self, ctx: C) -> <Post<M> as RangeRole>::Splits {
        position_splits(ctx.as_trace_context().patterns.iter(), *self)
    }
}

pub trait PatternSplits: Debug {
    type Pos;
    type Offsets;
    fn get(&self, pid: &PatternId) -> Option<Self::Pos>;
    fn offsets(&self) -> Self::Offsets;
}
//pub trait PatternSplitsRef<'a>: PatternSplits + Copy + 'a {
//    type Ref<'t>: Copy where Self: 't;
//    fn as_ref<'t>(&'t self) -> Self::Ref<'t> where Self: 't;
//}
//impl<'a> PatternSplits for OffsetSplitsRef<'a> {
//    type Pos = PatternSplitPos;
//    type Offsets = usize;
//    fn get(&self, pid: &PatternId) -> Option<Self::Pos> {
//        self.splits.get(pid).cloned()
//    }
//    fn offsets(&self) -> Self::Offsets {
//        self.offset.get()
//    }
//}
//impl<'a> PatternSplitsRef<'a> for OffsetSplitsRef<'a> {
//    type Ref<'t> = Self where Self: 't;
//    fn as_ref<'t>(&'t self) -> Self::Ref<'t> where Self: 't {
//        *self
//    }
//}
impl<'a> PatternSplits for OffsetSplits {
    type Pos = PatternSplitPos;
    type Offsets = usize;
    fn get(&self, pid: &PatternId) -> Option<Self::Pos> {
        self.splits.get(pid).cloned()
    }
    fn offsets(&self) -> Self::Offsets {
        self.offset.get()
    }
}
impl<'a> PatternSplits for &'a OffsetSplits {
    type Pos = PatternSplitPos;
    type Offsets = usize;
    fn get(&self, pid: &PatternId) -> Option<Self::Pos> {
        self.splits.get(pid).cloned()
    }
    fn offsets(&self) -> Self::Offsets {
        self.offset.get()
    }
}
//impl<'a> PatternSplitsRef<'a> for &'a OffsetSplits {
//    type Ref<'t> = Self where Self: 't;
//    fn as_ref<'t>(&'t self) -> Self::Ref<'t> where Self: 't {
//        *self
//    }
//}
impl<
    A: PatternSplits,
    B: PatternSplits,
> PatternSplits for (A, B) {
    type Pos = (A::Pos, B::Pos);
    type Offsets = (A::Offsets, B::Offsets);
    fn get(&self, pid: &PatternId) -> Option<Self::Pos> {
        self.0.get(pid).map(|a| {
            let b = self.1.get(pid).unwrap();
            (a, b)
        })
    }
    fn offsets(&self) -> Self::Offsets {
        (
            self.0.offsets(),
            self.1.offsets(),
        )
    }
}
//impl<
//    'a,
//    A: PatternSplitsRef<'a, Ref<'a> = OffsetSplitsRef<'a>> + 'a,
//    B: PatternSplitsRef<'a, Ref<'a> = OffsetSplitsRef<'a>> + 'a,
//> PatternSplitsRef<'a> for (A, B) {
//    type Ref<'t> = (OffsetSplitsRef<'t>, OffsetSplitsRef<'t>) where Self: 't;
//    fn as_ref<'t>(&'t self) -> Self::Ref<'t> where Self: 't {
//        (
//            self.0.as_ref(),
//            self.1.as_ref(),
//        )
//    }
//}