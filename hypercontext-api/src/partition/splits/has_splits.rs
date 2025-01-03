use super::{
    pos::SplitKind,
    PosSplits,
    PosSplitsOf, SubSplits,
};

use crate::split::cache::{
    position::SplitPositionCache,
    vertex::SplitVertexCache,
};
pub trait HasPosSplits: Sized {
    type Split: SplitKind;
    fn pos_splits(&self) -> &PosSplits<Self::Split>
    where
        for<'a> &'a Self: HasPosSplits<Split = Self::Split>,
        for<'a> &'a mut Self: HasPosSplits<Split = Self::Split>;
}

impl HasPosSplits for SplitVertexCache {
    type Split = SplitPositionCache;
    fn pos_splits(&self) -> &PosSplits<Self::Split> {
        &self.positions
    }
}

impl<S: HasPosSplits> HasPosSplits for &S {
    type Split = S::Split;
    fn pos_splits(&self) -> &PosSplits<Self::Split> {
        (**self).pos_splits()
    }
}

impl<S: HasPosSplits> HasPosSplits for &mut S {
    type Split = S::Split;
    fn pos_splits(&self) -> &PosSplits<Self::Split> {
        (**self).pos_splits()
    }
}

impl HasPosSplits for PosSplitsOf<SplitVertexCache> {
    type Split = <SplitVertexCache as HasPosSplits>::Split;
    fn pos_splits(&self) -> &PosSplits<Self::Split> {
        self
    }
}

pub trait HasSubSplits {
    fn sub_splits(&self) -> &SubSplits;
}

impl HasSubSplits for SubSplits {
    fn sub_splits(&self) -> &SubSplits {
        self
    }
}
//impl HasSubSplits for PosSplits<SplitVertexCache> {
//    fn sub_splits(&self) -> &SubSplits {
//        &self.into_iter().collect()
//    }
//}
