use crate::{
    graph::vertex::child::Child,
    interval::cache::PosKey,
    HashMap,
};
use std::fmt::Debug;

pub mod context;
pub mod joined;
pub mod partition;

pub trait SplitInner: Debug + Clone {}

impl<T: Debug + Clone> SplitInner for T {}

#[derive(Debug, Clone)]
pub struct Split<T: SplitInner = Child> {
    pub left: T,
    pub right: T,
}

impl<T: SplitInner> Split<T> {
    pub fn new(
        left: T,
        right: T,
    ) -> Self {
        Self { left, right }
    }
}

impl<I, T: SplitInner + Extend<I> + IntoIterator<Item = I>> Split<T> {
    pub fn infix(
        &mut self,
        mut inner: Split<T>,
    ) {
        self.left.extend(inner.left);
        inner.right.extend(self.right.clone());
        self.right = inner.right;
    }
}

pub type SplitMap = HashMap<PosKey, Split>;
//pub trait HasSplitMap {
//    fn split_map(&self) -> &SplitMap;
//}
//
//impl HasSplitMap for SplitMap {
//    fn split_map(&self) -> &SplitMap {
//        self
//    }
//}
//impl HasSplitMap for PosSplits<SplitVertexCache> {
//    fn split_map(&self) -> &SubSplits {
//        &self.into_iter().collect()
//    }
//}
