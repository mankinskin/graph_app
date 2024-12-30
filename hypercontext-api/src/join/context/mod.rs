use itertools::Itertools;
use linked_hash_set::LinkedHashSet;
use node::context::NodeJoinContext;

use crate::{
    graph::{
        vertex::{
            child::Child,
            has_vertex_index::HasVertexIndex, wide::Wide,
        },
        Hypergraph, HypergraphRef,
    },
    partition::splits::{
        HasSubSplits,
        SubSplits,
    },
    split::cache::{split::Split, SplitCache},
    traversal::{cache::key::SplitKey, fold::state::FoldState, traversable::TraversableMut}, HashMap,
};


pub mod node;
pub mod pattern;

#[derive(Debug)]
pub struct JoinContext<'a> {
    pub graph: <HypergraphRef as TraversableMut>::GuardMut<'a>,
    pub sub_splits: &'a SubSplits,
}

impl<'a> JoinContext<'a> {
    //
    pub fn new<SS: HasSubSplits>(
        graph: <HypergraphRef as TraversableMut>::GuardMut<'a>,
        sub_splits: &'a SS,
    ) -> Self {
        Self {
            graph,
            sub_splits: sub_splits.sub_splits(),
        }
    }
    pub fn node<'b>(
        &'b mut self,
        index: Child,
        split_cache: &'b SplitCache,
    ) -> NodeJoinContext<'a, 'b>
        where Self: 'a,
        'a: 'b,
    {
        NodeJoinContext::new(
            self,
            index,
            split_cache.entries.get(&index.vertex_index()).unwrap(),
        )
    }
    pub fn join_subgraph(
        &mut self,
        fold_state: FoldState,
    ) -> Child {
        let root = fold_state.root;
        let split_cache = SplitCache::new(self, fold_state);

        let final_splits = self.join_final_splits(&split_cache);

        let root_mode = split_cache.root_mode;
        let x = self
            //.join(&final_splits)
            .node(root, &split_cache)
            .join_root_partitions(root_mode);
        x
    }
    pub fn join_final_splits(
        &mut self,
        split_cache: &SplitCache,
    ) -> HashMap<SplitKey, Split> {
        let mut final_splits = HashMap::default();
        let keys = split_cache.leaves.iter().cloned().rev();
        let mut frontier: LinkedHashSet<SplitKey> = LinkedHashSet::from_iter(keys);
        while let Some(key) = {
            frontier.pop_front()
                .and_then(|key| (key.index != split_cache.root).then_some(key))
        } {
            if !final_splits.contains_key(&key) {
                let finals = self
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
            frontier.extend(top);
        }
        final_splits
    }

}
