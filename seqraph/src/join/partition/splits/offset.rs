use std::num::NonZeroUsize;

use crate::{
    join::partition::splits::SplitKind,
    split::VertexSplitPos,
};

#[derive(Debug, Clone)]
pub struct OffsetSplits {
    pub offset: NonZeroUsize,
    pub splits: VertexSplitPos,
}

//#[derive(Debug, Clone, Copy)]
//pub struct OffsetSplitsRef<'a> {
//    pub offset: NonZeroUsize,
//    pub splits: &'a VertexSplitPos,
//}
pub trait AsOffsetSplits: Clone {
    fn as_offset_splits(self) -> OffsetSplits;
}

impl AsOffsetSplits for OffsetSplits {
    fn as_offset_splits(self) -> OffsetSplits {
        self
    }
}

impl<'a> AsOffsetSplits for &'a OffsetSplits {
    fn as_offset_splits(self) -> OffsetSplits {
        self.clone()
    }
}

impl<'a, S: SplitKind> AsOffsetSplits for (&'a NonZeroUsize, S) {
    fn as_offset_splits(self) -> OffsetSplits {
        OffsetSplits {
            offset: self.0.clone(),
            splits: self.1.borrow().clone(),
        }
    }
}
//impl<'a, O: AsOffsetSplits<'a>> AsOffsetSplits<'a> for &'a O {
//    fn as_offset_splits<'t>(self) -> OffsetSplitsRef<'t> where 'a: 't {
//        (*self).as_offset_splits()
//    }
//}
//impl<'a> AsOffsetSplits<'a> for &'a SplitPositionCache {
//    fn as_offset_splits<'t>(self) -> OffsetSplitsRef<'t> where 'a: 't {
//        (*self).as_offset_splits()
//    }
//}
//impl<'a> AsOffsetSplits<'a> for &'a OffsetSplits {
//    fn as_offset_splits<'t>(self) -> &'t OffsetSplits where 'a: 't {
//        &OffsetSplits {
//            offset: self.offset,
//            splits: self.splits,
//        }
//    }
//}
//impl<'a> AsOffsetSplits<'a> for (&'a NonZeroUsize, &'a SplitPositionCache) {
//    fn as_offset_splits<'t>(self) -> OffsetSplitsRef<'t> where 'a: 't {
//        OffsetSplitsRef {
//            offset: self.0.clone(),
//            splits: self.1.borrow(),
//        }
//    }
//}
//impl<'a> AsOffsetSplits<'a> for (&'a NonZeroUsize, &'a VertexSplitPos) {
//    fn as_offset_splits<'t>(self) -> OffsetSplitsRef<'t> where 'a: 't {
//        OffsetSplitsRef {
//            offset: self.0.clone(),
//            splits: self.1,
//        }
//    }
//}
