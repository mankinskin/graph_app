use crate::split::{
    cache::vertex::SplitVertexCache,
    VertexSplitPos,
};
use std::{
    borrow::Borrow,
    fmt::Debug,
    num::NonZeroUsize,
};

use super::PosSplitOf;

#[derive(Debug, Clone)]
pub struct PosSplitContext<'a, S: SplitKind> {
    pub pos: &'a NonZeroUsize,
    pub split: &'a S,
}
impl<S: SplitKind> Copy for PosSplitContext<'_, S> {}
impl PosSplitContext<'_, PosSplitOf<SplitVertexCache>> {
    //pub fn fetch_split(
    //    &self,
    //    split_cache: &SplitCache,
    //) -> (SplitKey, Split) {
    //    self.split.
    //}
}
pub trait SplitKind: Borrow<VertexSplitPos> + Debug + Sized + Clone {}

impl<S: Borrow<VertexSplitPos> + Debug + Sized + Clone> SplitKind for S {}
