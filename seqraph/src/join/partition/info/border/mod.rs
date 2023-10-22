use crate::*;

pub mod perfect;
pub use perfect::*;

pub mod join;
pub use join::*;

pub mod trace;
pub use trace::*;

pub mod visit;
pub use visit::*;

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
        let offset = pattern_pre_ctx_width(pattern, pos.sub_index);
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

//pub trait PartitionBorders<'a, K: RangeRole, Ctx: AsPatternTraceContext<'a>>: VisitBorders<'a, K, Ctx> {
//}
//impl<'a, K: RangeRole<Borders<'a, Ctx>=Self>, Ctx: AsPatternTraceContext<'a>> PartitionBorders<'a, K, Ctx> for BorderInfo 
//    where BorderInfo: VisitBorders<'a, K, Ctx>,
//{
//}
//impl<'a, M: InVisitMode, Ctx: AsPatternTraceContext<'a>> PartitionBorders<'a, In<M>, Ctx> for (BorderInfo, BorderInfo)
//{
//}