use super::*;
use std::{
    ops::RangeBounds,
    slice::SliceIndex,
};

pub trait PatternRangeIndex:
    SliceIndex<[Child], Output = [Child]> + RangeBounds<usize> + Iterator<Item = usize> + Debug + Clone
{
}
impl<
        T: SliceIndex<[Child], Output = [Child]> + RangeBounds<usize> + Iterator<Item = usize> + Debug + Clone,
    > PatternRangeIndex for T
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