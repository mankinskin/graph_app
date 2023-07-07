use crate::*;

use super::ModePatternCtxOf;

pub trait BoolPerfect: Default + Debug + Clone {
    type Result: BorderPerfect<Boolean = Self>;
    fn then_some(self, pid: PatternId) -> Self::Result;
}
impl BoolPerfect for bool {
    type Result = Option<PatternId>;
    fn then_some(self, pid: PatternId) -> Self::Result {
        self.then_some(pid)
    }
}
impl BoolPerfect for (bool, bool) {
    type Result = (Option<PatternId>, Option<PatternId>);
    fn then_some(self, pid: PatternId) -> Self::Result {
        (
            self.0.then_some(pid),
            self.1.then_some(pid),
        )
    }
}
pub trait BorderPerfect: Default + Debug + Clone {
    type Boolean: BoolPerfect<Result=Self>;
    fn new(boolean: Self::Boolean, pid: PatternId) -> Self;
    fn fold_or(&mut self, other: Self);
    fn complete(&self) -> Option<PatternId>;
}
impl BorderPerfect for Option<PatternId> {
    type Boolean = bool;
    fn new(boolean: Self::Boolean, pid: PatternId) -> Self {
        boolean.then_some(pid)
    }
    fn fold_or(&mut self, other: Self) {
        *self = self.or(other);
    }
    fn complete(&self) -> Option<PatternId> {
        *self
    }
}
impl BorderPerfect for (Option<PatternId>, Option<PatternId>) {
    type Boolean = (bool, bool);
    fn new((a, b): Self::Boolean, pid: PatternId) -> Self {
        (
            a.then_some(pid),
            b.then_some(pid),
        )
    }
    fn fold_or(&mut self, other: Self) {
        self.0.fold_or(other.0);
        self.1.fold_or(other.1);
    }
    fn complete(&self) -> Option<PatternId> {
        self.0.complete().and_then(|pid|
            self.1.complete().filter(|i| *i == pid)
        )
    }
}

pub struct BorderInfo  {
    sub_index: usize,
    inner_offset: Option<NonZeroUsize>,
    outer_offset: Option<NonZeroUsize>,
}
impl BorderInfo {
    fn new(
        //cache: &SplitCache,
        pattern: &Pattern,
        pos: &PatternSplitPos,
    ) -> Self {
        let offset = pattern_offset(pattern, pos.sub_index);
        BorderInfo {
            sub_index: pos.sub_index,
            inner_offset: pos.inner_offset,
            outer_offset: NonZeroUsize::new(offset),
        }
    }
}

pub trait PartitionBorder<K: RangeRole>: Sized {
    fn perfect(&self) -> BooleanPerfectOf<K>;
    fn offsets(&self) -> OffsetsOf<K>;
}
impl<
    P: BorderPerfect<Boolean = bool>,
    K: RangeRole<Perfect = P, Offsets = NonZeroUsize>,
> PartitionBorder<K> for BorderInfo
{
    fn perfect(&self) -> BooleanPerfectOf<K> {
        self.inner_offset.is_none()
    }
    fn offsets(&self) -> OffsetsOf<K> {
        self.outer_offset.map(|o|
            self.inner_offset.map(|io|
                o.checked_add(io.get()).unwrap()
            ).unwrap_or(o)
        )
        .unwrap_or_else(|| self.inner_offset.unwrap())
    }
}
impl<M: InVisitMode> PartitionBorder<In<M>> for (BorderInfo, BorderInfo) {
    fn perfect(&self) -> BooleanPerfectOf<In<M>> {
        (
            <_ as PartitionBorder::<Pre<M>>>::perfect(&self.0),
            <_ as PartitionBorder::<Post<M>>>::perfect(&self.1),
        )
    }
    fn offsets(&self) -> OffsetsOf<In<M>> {
        (
            <_ as PartitionBorder::<Pre<M>>>::offsets(&self.0),
            <_ as PartitionBorder::<Post<M>>>::offsets(&self.1),
        )
    }
}
pub trait VisitBorders<'a, K: RangeRole>: Sized + PartitionBorder<K> {
    type Splits;
    fn make_borders(
        pattern: &Pattern,
        splits: &Self::Splits,
    ) -> Self;
    fn inner_range_offsets(&self, pattern: &Pattern) -> Option<OffsetsOf<K>>;
    fn inner_range(&self) -> RangeOf<K>;
    fn outer_range(&self) -> RangeOf<K>;

}
pub trait JoinBorders<'a, K: RangeRole<Mode = Join>>: VisitBorders<'a, K> {
    fn children(
        &self,
        ctx: &ModePatternCtxOf<'a, K>,
    ) -> Option<ModeChildrenOf<K>>;
}
pub trait TraceBorders<'a, K: RangeRole>: VisitBorders<'a, K> {
    fn inner_info(
        &self,
        ctx: &ModePatternCtxOf<'a, K>,
    ) -> Option<InnerRangeInfo<K>>;
    fn join_inner_info(
        self,
        ctx: &ModePatternCtxOf<'a, K>,
    ) -> Result<PatternRangeInfo<K>, Child> {
        let perfect = self.perfect();
        //let outer_range = left_border.outer_index()..right_border.outer_index();
        let range = self.outer_range();
        let offsets = self.offsets();
        let inner = self.inner_info(ctx);

        let ctx = ctx.as_pattern_trace_context();
        inner.as_ref().map(|inner| {
            let inner_pat = ctx.pattern.get(inner.range.clone()).unwrap();
            (inner_pat.len() != 1)
                .then(|| inner_pat.len().saturating_sub(1))
                .ok_or_else(|| inner_pat[0])
        })
        .unwrap_or(Ok(0))
        .map(|delta|
            PatternRangeInfo {
                pattern_id: ctx.pattern_id,
                info: RangeInfo {
                    inner_range: inner,
                    delta: delta,
                    offsets,
                    range,
                },
                perfect,
            }
        )
    }
}
impl<'a, K: RangeRole> TraceBorders<'a, K> for K::Borders<'a>
{
    fn inner_info(
        &self,
        ctx: &ModePatternCtxOf<'a, K>,
    ) -> Option<InnerRangeInfo<K>> {
        let pctx = ctx.as_pattern_trace_context();
        self.inner_range_offsets(pctx.pattern)
            .map(|offsets|
                InnerRangeInfo {
                    range: self.inner_range(),
                    offsets,
                    children: ModeOf::<K>::border_children(self, ctx),
                }
            )
    }
}
//impl<'a, K: RangeRole<Mode = Join>> TraceBorders<'a, K> for K::Borders<'a> {
//    fn inner_info(
//        &self,
//        ctx: &PatternJoinContext<'a>,
//    ) -> Option<InnerRangeInfo<K>> {
//        let pctx = ctx.as_pattern_join_context();
//        self.inner_range_offsets(pctx.pattern)
//            .map(|offsets|
//                InnerRangeInfo {
//                    range: self.inner_range(),
//                    offsets,
//                    children: self.children(ctx).expect("inner range needs children"),
//                }
//            )
//    }
//}
impl<'a, M: PostVisitMode> VisitBorders<'a, Post<M>> for BorderInfo {
    type Splits = PatternSplitPos;
    fn make_borders(
        pattern: &Pattern,
        splits: &Self::Splits,
    ) -> Self {
        Self::new(pattern, splits)
    }
    fn inner_range_offsets(&self, pattern: &Pattern) -> Option<OffsetsOf<Post<M>>> {
        (self.inner_offset.is_some() && pattern.len() - self.sub_index > 1)
            .then(|| {
                let w = pattern[self.sub_index].width();
                self.outer_offset.map(|o|
                    o.get() + w
                ).unwrap_or(w)
            })
            .and_then(NonZeroUsize::new)
    }
    fn inner_range(&self) -> RangeOf<Post<M>> {
        self.sub_index + self.inner_offset.is_some() as usize..
    }
    fn outer_range(&self) -> RangeOf<Post<M>> {
        self.sub_index..
    }
}
impl<'a> JoinBorders<'a, Post<Join>> for BorderInfo {
    fn children(
        &self,
        ctx: &PatternJoinContext<'a>,
    ) -> Option<ChildrenOf<Post<Join>>> {
        let ctx = ctx.as_pattern_join_context();
        self.inner_offset.map(|o|
            ctx.sub_splits.get(&SplitKey::new(ctx.pattern[self.sub_index], o)).unwrap().right
        )
    }
}

impl<'a, M: PreVisitMode> VisitBorders<'a, Pre<M>> for BorderInfo {
    type Splits = PatternSplitPos;
    fn make_borders(
        pattern: &Pattern,
        splits: &Self::Splits,
    ) -> Self {
        Self::new(pattern, splits)
    }
    fn inner_range_offsets(&self, _pattern: &Pattern) -> Option<OffsetsOf<Pre<M>>> {
        (self.inner_offset.is_some() && self.sub_index > 0)
            .then(|| self.outer_offset)
            .flatten()
    }
    fn inner_range(&self) -> RangeOf<Pre<M>> {
        0..self.sub_index
    }
    fn outer_range(&self) -> RangeOf<Pre<M>> {
        0..self.sub_index + self.inner_offset.is_some() as usize
    }
}
impl<'a> JoinBorders<'a, Pre<Join>> for BorderInfo {
    fn children(
        &self,
        ctx: &PatternJoinContext<'a>,
    ) -> Option<ChildrenOf<Pre<Join>>> {
        let ctx = ctx.as_pattern_join_context();
        self.inner_offset.map(|o|
            ctx.sub_splits.get(&SplitKey::new(ctx.pattern[self.sub_index], o)).unwrap().left
        )
    }
}
impl<'a, M: InVisitMode> VisitBorders<'a, In<M>> for (BorderInfo, BorderInfo) {
    type Splits = (
        <BorderInfo as VisitBorders<'a, Post<M>>>::Splits,
        <BorderInfo as VisitBorders<'a, Pre<M>>>::Splits,
    );
    fn make_borders(
        pattern: &Pattern,
        splits: &Self::Splits,
    ) -> Self {
        (
            BorderInfo::new(pattern, &splits.0),
            BorderInfo::new(pattern, &splits.1),
        )
    }
    fn inner_range_offsets(&self, pattern: &Pattern) -> Option<OffsetsOf<In<M>>> {
        let a = VisitBorders::<Post<M>>::inner_range_offsets(&self.0, pattern);
        let b = VisitBorders::<Pre<M>>::inner_range_offsets(&self.1, pattern);
        a.map(|lio|
            (
                lio,
                b.unwrap_or({
                    let w = pattern[self.1.sub_index].width();
                    let o = self.1.outer_offset.map(|o|
                        o.get() + w
                    ).unwrap_or(w);
                    NonZeroUsize::new(o).unwrap()
                })
            )
        )
        .or_else(||
            b.map(|rio|
                (
                    self.0.outer_offset.unwrap(),
                    rio,
                )
            )
        )
    }
    fn inner_range(&self) -> RangeOf<In<M>> {
        self.0.sub_index..self.1.sub_index
    }
    fn outer_range(&self) -> RangeOf<In<M>> {
        self.0.sub_index..self.1.sub_index
    }
}
impl<'a> JoinBorders<'a, In<Join>> for (BorderInfo, BorderInfo) {
    fn children(
        &self,
        ctx: &PatternJoinContext<'a>,
    ) -> Option<ChildrenOf<In<Join>>> {
        let ctx = ctx.as_pattern_join_context();
        let (lc, rc) = (ctx.pattern[self.0.sub_index], ctx.pattern[self.1.sub_index]);
        match (self.0.inner_offset, self.1.inner_offset) {
            (Some(l), Some(r)) => Some(InfixChildren::Both(
                ctx.sub_splits.get(&SplitKey::new(lc, l)).unwrap().right,
                ctx.sub_splits.get(&SplitKey::new(rc, r)).unwrap().left,
            )),
            (None, Some(r)) => Some(InfixChildren::Right(
                ctx.sub_splits.get(&SplitKey::new(rc, r)).unwrap().left,
            )),
            (Some(l), None) => Some(InfixChildren::Left(
                ctx.sub_splits.get(&SplitKey::new(lc, l)).unwrap().right,
            )),
            (None, None) => None,
        }
    }
}

//pub trait PartitionBorders<'a, K: RangeRole, Ctx: AsPatternTraceContext<'a>>: VisitBorders<'a, K, Ctx> {
//}
//impl<'a, K: RangeRole<Borders<'a, Ctx>=Self>, Ctx: AsPatternTraceContext<'a>> PartitionBorders<'a, K, Ctx> for BorderInfo 
//    where BorderInfo: VisitBorders<'a, K, Ctx>,
//{
//}
//impl<'a, M: InVisitMode, Ctx: AsPatternTraceContext<'a>> PartitionBorders<'a, In<M>, Ctx> for (BorderInfo, BorderInfo)
//{
//}