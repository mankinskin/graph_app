use std::iter::FromIterator;

use itertools::Itertools;
use linked_hash_set::LinkedHashSet;
use node::context::NodeJoinContext;

use crate::{
    graph::{
        vertex::{
            child::Child,
            wide::Wide,
        },
        Hypergraph,
        HypergraphRef,
    },
    interval::cache::{
        IntervalGraph,
        PosKey,
    },
    traversal::traversable::{
        TravKind,
        Traversable,
        TraversableMut,
    },
    HashMap,
};

use super::SplitMap;

pub mod node;
pub mod pattern;

#[derive(Debug)]
pub struct JoinContext {
    pub trav: HypergraphRef,
    //pub split_map: &'a SubSplits,
    pub interval: IntervalGraph,
}
#[derive(Debug)]
pub struct LockedJoinContext<'a> {
    pub trav: <HypergraphRef as TraversableMut>::GuardMut<'a>,
    //pub split_map: &'a SubSplits,
    pub interval: &'a IntervalGraph,
}

impl JoinContext {
    pub fn locked<'a: 'b, 'b>(&'a mut self) -> LockedJoinContext<'a> {
        LockedJoinContext {
            trav: self.trav.graph_mut(),
            interval: &self.interval,
        }
    }
    pub fn join_subgraph(&mut self) -> Child {
        let mut splits = HashMap::default();
        let leaves = self.interval.leaves.iter().cloned().rev();
        let mut frontier: LinkedHashSet<PosKey> = LinkedHashSet::from_iter(leaves);
        while let Some(key) = {
            frontier
                .pop_front()
                .and_then(|key| (key.index != self.interval.root).then_some(key))
        } {
            if !splits.contains_key(&key) {
                let partitions = self.node(key.index, &splits).join_partitions();

                for (key, split) in partitions {
                    splits.insert(key, split);
                }
            }
            let top = self
                .interval
                .expect(&key)
                .top
                .iter()
                .sorted_by(|a, b| a.index.width().cmp(&b.index.width()))
                .cloned();
            frontier.extend(top);
        }
        let joined = self
            .node(self.interval.root, &splits)
            .join_root_partitions();
        joined
    }

    fn node<'a: 'b, 'b>(
        &'a mut self,
        index: Child,
        splits: &'a SplitMap,
    ) -> NodeJoinContext<'a>
    where
        Self: 'a,
        'a: 'b,
    {
        NodeJoinContext {
            index,
            ctx: self.locked(),
            splits,
        }
    }
}

impl Traversable for JoinContext {
    type Kind = TravKind<Hypergraph>;
    type Guard<'g>
        = <HypergraphRef as Traversable>::Guard<'g>
    where
        Self: 'g;
    fn graph(&self) -> Self::Guard<'_> {
        self.trav.graph()
    }
}
impl TraversableMut for JoinContext {
    type GuardMut<'g>
        = <HypergraphRef as TraversableMut>::GuardMut<'g>
    where
        Self: 'g;
    fn graph_mut(&mut self) -> Self::GuardMut<'_> {
        self.trav.graph_mut()
    }
}
