use crate::*;

#[derive(Debug)]
pub struct OffsetSplits {
    pub offset: NonZeroUsize,
    pub splits: PatternSubSplits,
}
#[derive(Debug, Clone, Copy)]
pub struct OffsetSplitsRef<'a> {
    pub offset: NonZeroUsize,
    pub splits: &'a PatternSubSplits,
}
pub trait AsOffsetSplits<'a>: 'a {
    fn as_offset_splits<'t>(&'t self) -> OffsetSplitsRef<'t> where 'a: 't;
}
impl<'a, O: AsOffsetSplits<'a>> AsOffsetSplits<'a> for &'a O {
    fn as_offset_splits<'t>(&'t self) -> OffsetSplitsRef<'t> where 'a: 't {
        (*self).as_offset_splits()
    }
}
//impl<'a, K: RangeRole> AsOffsetSplits<'a> for K::Splits 
//    where K::Splits: 'a
//{
//    fn as_offset_splits<'t>(&'t self) -> OffsetSplitsRef<'t> where 'a: 't {
//        (*self).as_offset_splits()
//    }
//}
//impl<'a> From<OffsetSplits> for OffsetSplitsRef<'a> {
//    fn from(value: OffsetSplits) -> Self {
//        Self {
//            offset: value.offset,
//            splits: &value.splits,
//        }
//    }
//}
impl<'a> AsOffsetSplits<'a> for OffsetSplits {
    fn as_offset_splits<'t>(&'t self) -> OffsetSplitsRef<'t> where 'a: 't {
        OffsetSplitsRef {
            offset: self.offset,
            splits: &self.splits,
        }
    }
}
impl<'a, N: Borrow<NonZeroUsize> + 'a, S: Borrow<PatternSubSplits> + 'a> AsOffsetSplits<'a> for (N, &'a S) {
    fn as_offset_splits<'t>(&'t self) -> OffsetSplitsRef<'t> where 'a: 't {
        OffsetSplitsRef {
            offset: self.0.borrow().clone(),
            splits: self.1.borrow(),
        }
    }
}
impl<'a> AsOffsetSplits<'a> for OffsetSplitsRef<'a> {
    fn as_offset_splits<'t>(&'t self) -> OffsetSplitsRef<'t> where 'a: 't {
        *self
    }
}
