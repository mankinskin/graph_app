use std::{
    borrow::Borrow,
    collections::BTreeMap,
    fmt::Debug,
    num::NonZeroUsize,
};

use crate::{
    split::{
        cache::{
            position::SplitPositionCache,
            split::Split,
            vertex::SplitVertexCache,
        },
        VertexSplitPos,
    },
    HashMap,
};
use hypercontext_api::traversal::cache::key::SplitKey;

pub mod offset;

pub type PosSplits<S = SplitVertexCache> = BTreeMap<NonZeroUsize, <S as HasPosSplits>::Split>;
pub type PosSplitRef<'p, S = SplitVertexCache> = (&'p NonZeroUsize, &'p <S as HasPosSplits>::Split);

pub trait SplitKind: Borrow<VertexSplitPos> + Debug + Sized + Clone {}

impl<S: Borrow<VertexSplitPos> + Debug + Sized + Clone> SplitKind for S {}

pub trait HasPosSplits {
    type Split: SplitKind;
    fn pos_splits(&self) -> &PosSplits<Self>;
}

impl HasPosSplits for SplitVertexCache {
    type Split = SplitPositionCache;
    fn pos_splits(&self) -> &PosSplits<Self> {
        &self.positions
    }
}

impl<S: HasPosSplits> HasPosSplits for &S {
    type Split = S::Split;
    fn pos_splits(&self) -> &PosSplits<Self> {
        (**self).pos_splits()
    }
}

impl<S: HasPosSplits> HasPosSplits for &mut S {
    type Split = S::Split;
    fn pos_splits(&self) -> &PosSplits<Self> {
        (**self).pos_splits()
    }
}

impl HasPosSplits for PosSplits<SplitVertexCache> {
    type Split = <SplitVertexCache as HasPosSplits>::Split;
    fn pos_splits(&self) -> &PosSplits<Self> {
        self
    }
}

pub type SubSplits = HashMap<SplitKey, Split>;

pub trait HasSubSplits {
    fn sub_splits(&self) -> &SubSplits;
}

impl HasSubSplits for SubSplits {
    fn sub_splits(&self) -> &SubSplits {
        self
    }
}
//pub trait HasSubSplitsMut: HasPosSplits {
//    fn sub_splits_mut(&mut self) -> &mut PosSplits<Self>;
//}
//impl HasSubSplitsMut for SplitVertexCache {
//    fn sub_splits_mut(&mut self) -> &mut PosSplits<Self> {
//        &mut self.positions
//    }
//}
//impl<'a, S: HasSubSplitsMut> HasSubSplitsMut for &'a mut S {
//    fn sub_splits_mut(&mut self) -> &mut PosSplits<Self> {
//        self.sub_splits_mut()
//    }
//}
