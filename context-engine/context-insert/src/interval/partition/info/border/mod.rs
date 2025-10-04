use std::num::NonZeroUsize;

use perfect::*;

use crate::interval::partition::info::range::{
    mode::InVisitMode,
    role::{
        BooleanPerfectOf,
        In,
        OffsetsOf,
        Post,
        Pre,
        RangeRole,
    },
};
use context_trace::*;
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
        pos: &ChildTracePos,
    ) -> Self {
        let offset = End::inner_ctx_width(pattern, pos.sub_index);
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

impl<
    P: BorderPerfect<Boolean = bool>,
    R: RangeRole<Perfect = P, Offsets = NonZeroUsize>,
> PartitionBorder<R> for BorderInfo
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
