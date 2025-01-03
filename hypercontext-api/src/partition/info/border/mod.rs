use std::num::NonZeroUsize;

use perfect::*;

use crate::{
    graph::vertex::pattern::{
        pattern_pre_ctx_width,
        Pattern,
    },
    partition::info::range::{
        mode::InVisitMode,
        role::{
            BooleanPerfectOf,
            In,
            OffsetsOf,
            Post,
            Pre,
            RangeRole,
        },
    },
    split::PatternSplitPos,
};

pub mod perfect;

pub mod trace;

pub mod visit;

pub struct BorderInfo {
    pub sub_index: usize,
    pub inner_offset: Option<NonZeroUsize>,
    /// start offset of index with border
    pub start_offset: Option<NonZeroUsize>,
}

impl BorderInfo {
    fn new(
        pattern: &Pattern,
        pos: &PatternSplitPos,
    ) -> Self {
        let offset = pattern_pre_ctx_width(pattern, pos.sub_index);
        BorderInfo {
            sub_index: pos.sub_index,
            inner_offset: pos.inner_offset,
            start_offset: NonZeroUsize::new(offset),
        }
    }
}

pub trait PartitionBorder<R: RangeRole>: Sized {
    fn perfect(&self) -> BooleanPerfectOf<R>;
    fn offsets(&self) -> OffsetsOf<R>;
}

impl<P: BorderPerfect<Boolean = bool>, R: RangeRole<Perfect = P, Offsets = NonZeroUsize>>
    PartitionBorder<R> for BorderInfo
{
    fn perfect(&self) -> BooleanPerfectOf<R> {
        self.inner_offset.is_none()
    }
    fn offsets(&self) -> OffsetsOf<R> {
        self.start_offset
            .map(|o| {
                self.inner_offset
                    .map(|io| o.checked_add(io.get()).unwrap())
                    .unwrap_or(o)
            })
            .unwrap_or_else(|| self.inner_offset.unwrap())
    }
}

impl<M: InVisitMode> PartitionBorder<In<M>> for (BorderInfo, BorderInfo) {
    fn perfect(&self) -> BooleanPerfectOf<In<M>> {
        (
            <_ as PartitionBorder<Pre<M>>>::perfect(&self.0),
            <_ as PartitionBorder<Post<M>>>::perfect(&self.1),
        )
    }
    fn offsets(&self) -> OffsetsOf<In<M>> {
        (
            <_ as PartitionBorder<Pre<M>>>::offsets(&self.0),
            <_ as PartitionBorder<Post<M>>>::offsets(&self.1),
        )
    }
}

//impl<'a, R: RangeRole<Mode = Join>> TraceBorders<'a, R> for R::Borders<'a> {
//    fn inner_info(
//        &self,
//        ctx: &PatternJoinContext<'a>,
//    ) -> Option<InnerRangeInfo<R>> {
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

//pub trait PartitionBorders<'a, R: RangeRole, Ctx: GetPatternTraceContext<'a>>: VisitBorders<'a, R, Ctx> {
//}
//impl<'a, R: RangeRole<Borders<'a, Ctx>=Self>, Ctx: GetPatternTraceContext<'a>> PartitionBorders<'a, R, Ctx> for BorderInfo
//    where BorderInfo: VisitBorders<'a, R, Ctx>,
//{
//}
//impl<'a, M: InVisitMode, Ctx: GetPatternTraceContext<'a>> PartitionBorders<'a, In<M>, Ctx> for (BorderInfo, BorderInfo)
//{
//}
