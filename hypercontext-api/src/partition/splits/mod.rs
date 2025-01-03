use std::{
    collections::BTreeMap,
    fmt::Debug,
    num::NonZeroUsize,
};

use derive_more::derive::{
    Deref,
    DerefMut,
};
use has_splits::HasPosSplits;
use pos::{
    PosSplitContext,
    SplitKind,
};

use crate::{
    split::cache::split::Split,
    traversal::cache::key::SplitKey,
    HashMap,
};

pub mod has_splits;
pub mod offset;
pub mod pos;

pub type PosSplitsOf<S> = PosSplits<PosSplitOf<S>>;
pub type PosSplitOf<S> = <S as HasPosSplits>::Split;

pub type SubSplits = HashMap<SplitKey, Split>;

#[derive(Debug, Default, Clone, PartialEq, Eq, Deref, DerefMut)]
pub struct PosSplits<S: SplitKind> {
    pub splits: BTreeMap<NonZeroUsize, S>,
}
impl<'a, S: SplitKind> From<(&'a NonZeroUsize, &'a S)> for PosSplitContext<'a, S> {
    fn from(item: (&'a NonZeroUsize, &'a S)) -> Self {
        Self {
            pos: item.0,
            split: item.1,
        }
    }
}
impl<S: SplitKind> FromIterator<(NonZeroUsize, S)> for PosSplits<S> {
    fn from_iter<I: IntoIterator<Item = (NonZeroUsize, S)>>(iter: I) -> Self {
        Self {
            splits: BTreeMap::from_iter(iter),
        }
    }
}
