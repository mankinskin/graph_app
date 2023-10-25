use crate::*;

pub mod offset;
pub use offset::*;

pub type PosSplits<S=SplitVertexCache> = BTreeMap<NonZeroUsize, <S as HasPosSplits>::Split>;
pub type PosSplitRef<'p, S=SplitVertexCache> = (&'p NonZeroUsize, &'p <S as HasPosSplits>::Split);

pub trait SplitKind: Borrow<VertexSplitPos> + Debug + Sized + Clone {}
impl<'a, S: Borrow<VertexSplitPos> + Debug + Sized + 'a + Clone> SplitKind for S
{}

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
impl<'a, S: HasPosSplits> HasPosSplits for &'a S {
    type Split = S::Split;
    fn pos_splits(&self) -> &PosSplits<Self> {
        (**self).pos_splits()
    }
}
impl<'a, S: HasPosSplits> HasPosSplits for &'a mut S {
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