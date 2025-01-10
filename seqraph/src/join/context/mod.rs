use std::iter::FromIterator;

use itertools::Itertools;
use linked_hash_set::LinkedHashSet;
use node::context::NodeJoinContext;

use hypercontext_api::{
    graph::{
        vertex::{
            child::Child,
            wide::Wide,
        }, Hypergraph, HypergraphRef
    },
    split::cache::{split::Split, SplitCache},
    traversal::{cache::key::SplitKey, fold::state::FoldState, traversable::{TravKind, Traversable, TraversableMut}}, HashMap,
};


pub mod node;
pub mod pattern;

#[derive(Debug)]
pub struct JoinContext {
    pub trav: HypergraphRef,
    //pub sub_splits: &'a SubSplits,
    pub split_cache: SplitCache,
    pub root: Child,
}
#[derive(Debug)]
pub struct LockedJoinContext<'a> {
    pub trav: <HypergraphRef as TraversableMut>::GuardMut<'a>,
    //pub sub_splits: &'a SubSplits,
    pub split_cache: &'a SplitCache,
    pub root: Child,
}

impl JoinContext {
    //
    //pub fn new<SS: HasSubSplits>(
    //    graph: <HypergraphRef as TraversableMut>::GuardMut<'a>,
    //    sub_splits: &'a SS,
    //) -> Self {
    //    Self {
    //        graph,
    //        sub_splits: sub_splits.sub_splits(),
    //    }
    //}
    pub fn new(
        mut trav: HypergraphRef,
        fold_state: &mut FoldState,
    ) -> Self {
        let root = fold_state.root;
        let split_cache = SplitCache::new(&mut trav.graph_mut(), fold_state);
        Self {
            root,
            trav,
            //sub_splits: sub_splits.sub_splits(),
            split_cache
        }
    }
    pub fn locked<'a: 'b, 'b>(
        &'a mut self,
    ) -> LockedJoinContext<'a> {
        LockedJoinContext {
            trav: self.trav.graph_mut(),
            split_cache: &self.split_cache,
            root: self.root,
            //_ty: Default::default(),
        }
    }
    pub fn join_subgraph(
        &mut self,
    ) -> Child {
        let finished_splits = self.join_splits();
        self.join_root(finished_splits)
    }
    pub fn join_root(
        &mut self,
        finished_splits: HashMap<SplitKey, Split>,
    ) -> Child {
        self.node(self.root, &finished_splits)
            .join_root_partitions()
    }
    pub fn node<'a: 'b, 'b>(
        &'a mut self,
        index: Child,
        finished_splits: &'a HashMap<SplitKey, Split>
    ) -> NodeJoinContext<'a>
        where Self: 'a,
              'a: 'b
    {
        NodeJoinContext {
            index,
            ctx: self.locked(),
            finished_splits
        }
    }
    pub fn join_splits(
        &mut self,
    ) -> HashMap<SplitKey, Split> {
        let mut finished_splits = HashMap::default();
        let keys = self.split_cache.leaves.iter().cloned().rev();
        let mut frontier: LinkedHashSet<SplitKey> = LinkedHashSet::from_iter(keys);
        while let Some(key) = {
            frontier.pop_front()
                .and_then(|key| (key.index != self.split_cache.root).then_some(key))
        } {
            if !finished_splits.contains_key(&key) {
                let partitions = self
                    .node(key.index, &finished_splits)
                    .join_partitions();

                for (key, split) in partitions {
                    finished_splits.insert(key, split);
                }
            }
            let top = self.split_cache
                .expect(&key)
                .top
                .iter()
                .sorted_by(|a, b| a.index.width().cmp(&b.index.width()))
                .cloned();
            frontier.extend(top);
        }
        finished_splits
    }

}

impl Traversable for JoinContext {
    type Kind = TravKind<Hypergraph>;
    type Guard<'g> = <HypergraphRef as Traversable>::Guard<'g> where Self: 'g;
    fn graph(&self) -> Self::Guard<'_> {
        self.trav.graph()
    }
}
impl TraversableMut for JoinContext {
    type GuardMut<'g> = <HypergraphRef as TraversableMut>::GuardMut<'g> where Self: 'g;
    fn graph_mut(&mut self) -> Self::GuardMut<'_> {
        self.trav.graph_mut()
    }
}