use std::iter::FromIterator;

use derive_more::{
    Deref,
    DerefMut,
};
use linked_hash_set::LinkedHashSet;

use crate::traversal::cache::key::SplitKey;

#[derive(Debug, Default, Deref, DerefMut)]
pub struct SplitFrontier {
    pub queue: LinkedHashSet<SplitKey>,
}

impl SplitFrontier {
    pub fn new(keys: impl IntoIterator<Item = SplitKey>) -> Self {
        Self {
            queue: LinkedHashSet::from_iter(keys),
        }
    }
}

impl Extend<SplitKey> for SplitFrontier {
    fn extend<T: IntoIterator<Item = SplitKey>>(
        &mut self,
        iter: T,
    ) {
        self.queue.extend(iter)
    }
}
