use crate::*;

pub trait VisitBorders<'a, K: RangeRole>: Sized + PartitionBorder<K> {
    type Splits;
    fn info_border(
        pattern: &Pattern,
        splits: &Self::Splits,
    ) -> Self;
    fn inner_range_offsets(&self, pattern: &Pattern) -> Option<OffsetsOf<K>>;
    fn inner_range(&self) -> RangeOf<K>;
    fn outer_range(&self) -> RangeOf<K>;
}
impl<'a, M: PostVisitMode> VisitBorders<'a, Post<M>> for BorderInfo {
    type Splits = PatternSplitPos;
    fn info_border(
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
impl<'a, M: PreVisitMode> VisitBorders<'a, Pre<M>> for BorderInfo {
    type Splits = PatternSplitPos;
    fn info_border(
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
impl<'a, M: InVisitMode> VisitBorders<'a, In<M>> for (BorderInfo, BorderInfo) {
    type Splits = (
        <BorderInfo as VisitBorders<'a, Post<M>>>::Splits,
        <BorderInfo as VisitBorders<'a, Pre<M>>>::Splits,
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