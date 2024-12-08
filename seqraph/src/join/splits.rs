use std::iter::FromIterator;

use derive_more::{Deref, DerefMut};

use itertools::Itertools;
use linked_hash_set::LinkedHashSet;

use crate::{graph::vertex::{child::Child, wide::Wide}, join::JoinContext, split::cache::{split::Split, SplitCache}, traversal::cache::key::SplitKey, HashMap};


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
    pub fn join_final_splits(&self, root: Child, split_cache: &SplitCache) -> HashMap<SplitKey, Split> {

        let mut final_splits = HashMap::default();
        while let Some(key) = {
            self.pop_front()
                .and_then(|key| (key.index != root).then_some(key))
        } {
            if !final_splits.contains_key(&key) {
                let finals = JoinContext::new(self.graph_mut(), &final_splits)
                        .node(key.index, split_cache)
                        .join_partitions();

                for (key, split) in finals {
                    final_splits.insert(key, split);
                }
            }
            let top = split_cache
                .expect(&key)
                .top
                .iter()
                .sorted_by(|a, b| a.index.width().cmp(&b.index.width()))
                .cloned();
            self.extend(top);
        }
        final_splits
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