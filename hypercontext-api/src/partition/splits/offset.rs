use std::num::NonZeroUsize;

use crate::{
    partition::splits::pos::SplitKind,
    split::VertexSplitPos,
};

use super::pos::PosSplitContext;

#[derive(Debug, Clone)]
pub struct OffsetSplit {
    pub offset: NonZeroUsize,
    pub splits: VertexSplitPos,
}

//#[derive(Debug, Clone, Copy)]
//pub struct OffsetSplitRef<'a> {
//    pub offset: NonZeroUsize,
//    pub splits: &'a VertexSplitPos,
//}
pub trait ToOffsetSplit: Clone {
    fn to_offset_splits(self) -> OffsetSplit;
}

impl ToOffsetSplit for OffsetSplit {
    fn to_offset_splits(self) -> OffsetSplit {
        self
    }
}

impl ToOffsetSplit for &OffsetSplit {
    fn to_offset_splits(self) -> OffsetSplit {
        self.clone()
    }
}

impl<'a, S: SplitKind> ToOffsetSplit for PosSplitContext<'a, S> {
    fn to_offset_splits(self) -> OffsetSplit {
        OffsetSplit {
            offset: *self.pos,
            splits: self.split.borrow().clone(),
        }
    }
}
impl<'a, S: SplitKind> ToOffsetSplit for (NonZeroUsize, S) {
    fn to_offset_splits(self) -> OffsetSplit {
        OffsetSplit {
            offset: self.0,
            splits: self.1.borrow().clone(),
        }
    }
}
impl<'a, S: SplitKind> From<(NonZeroUsize, S)> for OffsetSplit {
    fn from(item: (NonZeroUsize, S)) -> OffsetSplit {
        OffsetSplit {
            offset: item.0,
            splits: item.1.borrow().clone(),
        }
    }
}
impl<'a, S: SplitKind> From<(&'a NonZeroUsize, &'a S)> for OffsetSplit {
    fn from(item: (&'a NonZeroUsize, &'a S)) -> OffsetSplit {
        OffsetSplit {
            offset: *item.0,
            splits: (*item.1).borrow().clone(),
        }
    }
}
//impl<'a, O: AsOffsetSplit<'a>> AsOffsetSplit<'a> for &'a O {
//    fn as_offset_splits<'t>(self) -> OffsetSplitRef<'t> where 'a: 't {
//        (*self).as_offset_splits()
//    }
//}
//impl<'a> AsOffsetSplit<'a> for &'a SplitPositionCache {
//    fn as_offset_splits<'t>(self) -> OffsetSplitRef<'t> where 'a: 't {
//        (*self).as_offset_splits()
//    }
//}
//impl<'a> AsOffsetSplit<'a> for &'a OffsetSplit {
//    fn as_offset_splits<'t>(self) -> &'t OffsetSplit where 'a: 't {
//        &OffsetSplit {
//            offset: self.offset,
//            splits: self.splits,
//        }
//    }
//}
//impl<'a> AsOffsetSplit<'a> for (&'a NonZeroUsize, &'a SplitPositionCache) {
//    fn as_offset_splits<'t>(self) -> OffsetSplitRef<'t> where 'a: 't {
//        OffsetSplitRef {
//            offset: self.0.clone(),
//            splits: self.1.borrow(),
//        }
//    }
//}
//impl<'a> AsOffsetSplit<'a> for (&'a NonZeroUsize, &'a VertexSplitPos) {
//    fn as_offset_splits<'t>(self) -> OffsetSplitRef<'t> where 'a: 't {
//        OffsetSplitRef {
//            offset: self.0.clone(),
//            splits: self.1,
//        }
//    }
//}
