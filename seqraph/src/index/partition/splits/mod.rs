use crate::*;

pub mod offset;
pub use offset::*;
pub mod partition;
pub use partition::*;

pub trait HasSubSplits {
    type Split: Borrow<PatternSubSplits>;
    fn sub_splits(&self) -> &BTreeMap<NonZeroUsize, Self::Split>;
}
impl HasSubSplits for SplitVertexCache {
    type Split = SplitPositionCache;
    fn sub_splits(&self) -> &BTreeMap<NonZeroUsize, Self::Split> {
        &self.positions
    }
}
pub trait HasSubSplitsMut: HasSubSplits {
    fn sub_splits_mut(&mut self) -> &mut BTreeMap<NonZeroUsize, Self::Split>;
}
impl HasSubSplitsMut for SplitVertexCache {
    fn sub_splits_mut(&mut self) -> &mut BTreeMap<NonZeroUsize, Self::Split> {
        &mut self.positions
    }
}
//impl<C: Indexed> HasSubSplits for C {
//    type Split = SplitPositionCache;
//    fn sub_splits<'s>(&self) -> &'s BTreeMap<NonZeroUsize, Self::Split> {
//        &cache.entries.get(&self.index()).unwrap().positions
//    }
//}