use crate::*;

pub mod offset;
pub use offset::*;

pub type SplitPos<S> = BTreeMap<NonZeroUsize, <S as HasSplitPos>::Split>;

pub trait SplitKind: Borrow<VertexSplitPos> + Debug + Sized + Clone {}
impl<'a, S: Borrow<VertexSplitPos> + Debug + Sized + 'a + Clone> SplitKind for S
{}

pub trait HasSplitPos {
    type Split: SplitKind;
    fn split_pos(&self) -> &SplitPos<Self>;
}
impl HasSplitPos for SplitVertexCache {
    type Split = SplitPositionCache;
    fn split_pos(&self) -> &SplitPos<Self> {
        &self.positions
    }
}
impl<'a, S: HasSplitPos> HasSplitPos for &'a S {
    type Split = S::Split;
    fn split_pos(&self) -> &SplitPos<Self> {
        (**self).split_pos()
    }
}
impl<'a, S: HasSplitPos> HasSplitPos for &'a mut S {
    type Split = S::Split;
    fn split_pos(&self) -> &SplitPos<Self> {
        (**self).split_pos()
    }
}
impl HasSplitPos for SplitPos<SplitVertexCache> {
    type Split = <SplitVertexCache as HasSplitPos>::Split;
    fn split_pos(&self) -> &SplitPos<Self> {
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
//pub trait HasSubSplitsMut: HasSplitPos {
//    fn sub_splits_mut(&mut self) -> &mut SplitPos<Self>;
//}
//impl HasSubSplitsMut for SplitVertexCache {
//    fn sub_splits_mut(&mut self) -> &mut SplitPos<Self> {
//        &mut self.positions
//    }
//}
//impl<'a, S: HasSubSplitsMut> HasSubSplitsMut for &'a mut S {
//    fn sub_splits_mut(&mut self) -> &mut SplitPos<Self> {
//        self.sub_splits_mut()
//    }
//}