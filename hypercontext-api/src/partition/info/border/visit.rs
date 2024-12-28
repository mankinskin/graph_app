use std::num::NonZeroUsize;

use crate::{
    join::partition::info::{
        border::{
            BorderInfo,
            PartitionBorder,
        },
        range::role::{
            In,
            InVisitMode,
            OffsetsOf,
            Post,
            PostVisitMode,
            Pre,
            PreVisitMode,
            RangeOf,
            RangeRole,
        },
    },
    split::PatternSplitPos,
};
use hypercontext_api::graph::vertex::{
    pattern::Pattern,
    wide::Wide,
};

pub trait VisitBorders<K: RangeRole>: Sized + PartitionBorder<K> {
    type Splits;
    fn info_border(
        pattern: &Pattern,
        splits: &Self::Splits,
    ) -> Self;
    fn inner_range_offsets(
        &self,
        pattern: &Pattern,
    ) -> Option<OffsetsOf<K>>;
    fn inner_range(&self) -> RangeOf<K>;
    fn outer_range(&self) -> RangeOf<K>;
}

impl<M: PostVisitMode> VisitBorders<Post<M>> for BorderInfo {
    type Splits = PatternSplitPos;
    fn info_border(
        pattern: &Pattern,
        splits: &Self::Splits,
    ) -> Self {
        Self::new(pattern, splits)
    }
    fn inner_range_offsets(
        &self,
        pattern: &Pattern,
    ) -> Option<OffsetsOf<Post<M>>> {
        (self.inner_offset.is_some() && pattern.len() - self.sub_index > 1)
            .then(|| {
                let w = pattern[self.sub_index].width();
                self.start_offset.map(|o| o.get() + w).unwrap_or(w)
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

impl<M: PreVisitMode> VisitBorders<Pre<M>> for BorderInfo {
    type Splits = PatternSplitPos;
    fn info_border(
        pattern: &Pattern,
        splits: &Self::Splits,
    ) -> Self {
        Self::new(pattern, splits)
    }
    fn inner_range_offsets(
        &self,
        _pattern: &Pattern,
    ) -> Option<OffsetsOf<Pre<M>>> {
        (self.inner_offset.is_some() && self.sub_index > 0)
            .then_some(self.start_offset)
            .flatten()
    }
    fn inner_range(&self) -> RangeOf<Pre<M>> {
        0..self.sub_index
    }
    fn outer_range(&self) -> RangeOf<Pre<M>> {
        0..self.sub_index + self.inner_offset.is_some() as usize
    }
}

impl<M: InVisitMode> VisitBorders<In<M>> for (BorderInfo, BorderInfo) {
    type Splits = (
        <BorderInfo as VisitBorders<Post<M>>>::Splits,
        <BorderInfo as VisitBorders<Pre<M>>>::Splits,
    );
    fn info_border(
        pattern: &Pattern,
        splits: &Self::Splits,
    ) -> Self {
        (
            BorderInfo::new(pattern, &splits.0),
            BorderInfo::new(pattern, &splits.1),
        )
    }
    fn inner_range_offsets(
        &self,
        pattern: &Pattern,
    ) -> Option<OffsetsOf<In<M>>> {
        let a = VisitBorders::<Post<M>>::inner_range_offsets(&self.0, pattern);
        let b = VisitBorders::<Pre<M>>::inner_range_offsets(&self.1, pattern);
        let r = match (a, b) {
            (Some(lio), Some(rio)) => Some((lio, rio)),
            (Some(lio), None) => Some((lio, {
                let w = pattern[self.1.sub_index].width();
                let o = self.1.start_offset.unwrap().get() + w;
                NonZeroUsize::new(o).unwrap()
            })),
            (None, Some(rio)) => Some((self.0.start_offset.unwrap(), rio)),
            (None, None) => None,
        };
        r.filter(|(l, r)| l != r)
    }
    fn inner_range(&self) -> RangeOf<In<M>> {
        self.0.sub_index..self.1.sub_index
    }
    fn outer_range(&self) -> RangeOf<In<M>> {
        self.0.sub_index..self.1.sub_index + self.1.inner_offset.is_some() as usize
    }
}
