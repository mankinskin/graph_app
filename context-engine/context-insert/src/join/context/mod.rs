use itertools::Itertools;
use linked_hash_set::LinkedHashSet;
use node::context::NodeJoinContext;

use crate::{
    interval::IntervalGraph,
    split::{
        SplitMap,
        cache::position::PosKey,
    },
};
use context_trace{
    HashMap,
    graph::{
        Hypergraph,
        HypergraphRef,
        vertex::{
            child::Child,
            wide::Wide,
        },
    },
    traversal::has_graph::{
        TravKind,
        HasGraph,
        HasGraphMut,
    },
};

pub mod node;
pub mod pattern;

//pub trait VertexJoin: HasGraphMut {
//    fn join_prefix(
//        &mut self,
//        vertex: impl HasVertexDataMut,
//        end_bound: usize,
//    ) {
//        JoinContext {
//            self,
//        }
//    }
//    fn join_infix(
//        &mut self,
//        vertex: impl HasVertexDataMut,
//        start_bound: usize,
//        end_bound: usize,
//    ) {
//    }
//    fn join_postfix(
//        &mut self,
//        vertex: impl HasVertexDataMut,
//        start_bound: usize,
//    ) {
//    }
//}

#[derive(Debug)]
pub struct JoinContext
{
    pub trav: HypergraphRef,
    //pub split_map: &'a SubSplits,
    pub interval: IntervalGraph,
}
#[derive(Debug)]
pub struct LockedJoinContext<'a>
{
    pub trav: <HypergraphRef as HasGraphMut>::GuardMut<'a>,
    //pub split_map: &'a SubSplits,
    pub interval: &'a IntervalGraph,
}

impl JoinContext
{
    pub fn locked<'a: 'b, 'b>(&'a mut self) -> LockedJoinContext<'a>
    {
        LockedJoinContext {
            trav: self.trav.graph_mut(),
            interval: &self.interval,
        }
    }
    pub fn join_subgraph(&mut self) -> Child
    {
        let mut splits = HashMap::default();
        let leaves = self.interval.states.leaves.iter().cloned().rev();
        let mut frontier: LinkedHashSet<PosKey> =
            LinkedHashSet::from_iter(leaves);
        while let Some(key) = {
            frontier.pop_front().and_then(|key| {
                (key.index != self.interval.root).then_some(key)
            })
        }
        {
            if !splits.contains_key(&key)
            {
                let partitions =
                    self.node(key.index, &splits).join_partitions();

                for (key, split) in partitions
                {
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

impl HasGraph for JoinContext
{
    type Kind = TravKind<Hypergraph>;
    type Guard<'g>
        = <HypergraphRef as HasGraph>::Guard<'g>
    where
        Self: 'g;
    fn graph(&self) -> Self::Guard<'_>
    {
        self.trav.graph()
    }
}
impl HasGraphMut for JoinContext
{
    type GuardMut<'g>
        = <HypergraphRef as HasGraphMut>::GuardMut<'g>
    where
        Self: 'g;
    fn graph_mut(&mut self) -> Self::GuardMut<'_>
    {
        self.trav.graph_mut()
    }
}
