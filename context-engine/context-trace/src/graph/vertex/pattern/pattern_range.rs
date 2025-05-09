use std::{
    fmt::Debug,
    ops::RangeBounds,
    slice::SliceIndex,
};

use super::{
    super::child::Child,
    Pattern,
};
use crate::graph::{
    getters::ErrorReason,
    vertex::PatternId,
};

pub fn get_child_pattern_range<'a, R: PatternRangeIndex>(
    id: &PatternId,
    p: &'a Pattern,
    range: R,
) -> Result<&'a <R as SliceIndex<[Child]>>::Output, ErrorReason> {
    p.get(range.clone())
        .ok_or_else(|| ErrorReason::InvalidPatternRange(*id, p.clone(), format!("{:#?}", range)))
}

pub trait PatternRangeIndex<T = Child>:
    SliceIndex<[T], Output = [T]>
    + RangeBounds<usize>
    + Iterator<Item = usize>
    + Debug
    + Clone
    + Send
    + Sync
{
}

impl<
        T,
        R: SliceIndex<[T], Output = [T]>
            + RangeBounds<usize>
            + Iterator<Item = usize>
            + Debug
            + Clone
            + Send
            + Sync,
    > PatternRangeIndex<T> for R
{
}
pub trait StartInclusive {
    fn start(&self) -> usize;
}
impl StartInclusive for std::ops::RangeInclusive<usize> {
    fn start(&self) -> usize {
        *self.start()
    }
}
impl StartInclusive for std::ops::RangeTo<usize> {
    fn start(&self) -> usize {
        0
    }
}
impl StartInclusive for std::ops::RangeFrom<usize> {
    fn start(&self) -> usize {
        self.start
    }
}
impl StartInclusive for std::ops::Range<usize> {
    fn start(&self) -> usize {
        self.start
    }
}
pub trait EndInclusive {
    fn end(&self) -> usize;
}
impl EndInclusive for std::ops::RangeInclusive<usize> {
    fn end(&self) -> usize {
        *self.end()
    }
}
impl EndInclusive for std::ops::RangeToInclusive<usize> {
    fn end(&self) -> usize {
        self.end
    }
}
impl EndInclusive for std::ops::Range<usize> {
    fn end(&self) -> usize {
        self.end
    }
}
