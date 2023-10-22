use crate::*;

pub mod joined;
pub use joined::*;
pub mod delta;
pub use delta::*;
pub mod context;
pub use context::*;
pub mod partition;
pub use partition::*;

#[derive(Debug, Default, Deref, DerefMut)]
pub struct SplitFrontier {
    pub queue: LinkedHashSet<SplitKey>,
}
impl SplitFrontier {
    pub fn new(keys: impl IntoIterator<Item=SplitKey>) -> Self {
        Self {
            queue: LinkedHashSet::from_iter(keys),
        }
    }
}
impl Extend<SplitKey> for SplitFrontier {
    fn extend<T: IntoIterator<Item = SplitKey>>(&mut self, iter: T) {
        self.queue.extend(iter)
    }
}
impl Indexer {
    pub fn join_subgraph(
        &mut self,
        fold_state: FoldState,
    ) -> Child {
        let root = fold_state.root;
        let split_cache = SplitCache::new(self, fold_state);

        let mut frontier = SplitFrontier::new(
            split_cache.leaves.iter().cloned().rev()
        );
        let mut final_splits = HashMap::default();
        while let Some(key) = {
            frontier.pop_front()
                .and_then(|key|
                    (key.index != root).then(|| key)
                )
        } {
            if final_splits.get(&key).is_none() {
                let finals = {
                    JoinContext::new(
                        self.graph_mut(),
                        &final_splits,
                    )
                    .node(key.index, &split_cache)
                    .join_node_partitions()
                };

                for (key, split) in finals {
                    final_splits.insert(key, split);
                }
            }
            let top = 
                split_cache.expect(&key).top.iter()
                    .sorted_by(|a, b| a.index.width().cmp(&b.index.width()))
                    .cloned();
            frontier.extend(top);
        }
        let root_mode = split_cache.root_mode;
        let c = JoinContext::new(
                self.graph_mut(),
                &final_splits,
            )
            .node(root, &split_cache)
            .join_root_partitions(root_mode);
        c
    }
}