use context::{node::kind::JoinKind, JoinContext};
use itertools::Itertools;

use crate::{graph::vertex::{wide::Wide, child::Child}, split::{cache::{split::Split, SplitCache}, frontier::SplitFrontier}, traversal::{cache::key::SplitKey, traversable::TraversableMut}, HashMap};

pub mod context;
pub mod joined;

pub mod partition;

impl SplitFrontier {
    pub fn join_final_splits<K: JoinKind>(
        &mut self,
        trav: &mut K::Trav,
        root: Child,
        split_cache: &SplitCache,
    ) -> HashMap<SplitKey, Split> {
        let mut final_splits = HashMap::default();
        while let Some(key) = {
            self.pop_front()
                .and_then(|key| (key.index != root).then_some(key))
        } {
            if !final_splits.contains_key(&key) {
                let finals = JoinContext::<K>::new(trav.graph_mut(), &final_splits)
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