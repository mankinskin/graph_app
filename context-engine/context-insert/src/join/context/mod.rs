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
use context_trace::{
    graph::{
        Hypergraph,
        HypergraphRef,
        vertex::{
            child::Child,
            wide::Wide,
        },
    },
    trace::has_graph::{
        HasGraph,
        HasGraphMut,
        TravKind,
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

pub struct FrontierIterator {
    frontier: LinkedHashSet<PosKey>,
    interval: IntervalGraph,
}
impl Iterator for FrontierIterator {
    type Item = Option<PosKey>;
    fn next(&mut self) -> Option<Self::Item> {
        match self.frontier.pop_front() {
            Some(key) =>
                Some(match (key.index != self.interval.root).then_some(key) {
                    Some(key) => {
                        let top = self
                            .interval
                            .expect(&key)
                            .top
                            .iter()
                            .sorted_by(|a, b| {
                                a.index.width().cmp(&b.index.width())
                            })
                            .cloned();
                        self.frontier.extend(top);
                        Some(key)
                    },
                    None => None,
                }),
            None => None,
        }
    }
}
pub struct FrontierSplitIterator {
    frontier: FrontierIterator,
    splits: SplitMap,
    trav: HypergraphRef,
}
impl FrontierSplitIterator {
    pub fn locked<'a: 'b, 'b>(&'a mut self) -> LockedJoinContext<'a> {
        LockedJoinContext {
            trav: self.trav.graph_mut(),
            interval: &self.frontier.interval,
            splits: &self.splits,
        }
    }

    fn node<'a: 'b, 'b>(
        &'a mut self,
        index: Child,
    ) -> NodeJoinContext<'a>
    where
        Self: 'a,
        'a: 'b,
    {
        NodeJoinContext {
            index,
            ctx: self.locked(),
        }
    }
}
impl Iterator for FrontierSplitIterator {
    type Item = Option<Child>;
    fn next(&mut self) -> Option<Self::Item> {
        Some(match self.frontier.next() {
            Some(Some(key)) => {
                if !self.splits.contains_key(&key) {
                    let partitions = self.node(key.index).join_partitions();

                    for (key, split) in partitions {
                        self.splits.insert(key, split);
                    }
                }
                None
            },
            Some(None) => None,
            None => Some(
                self.node(self.frontier.interval.root)
                    .join_root_partitions(),
            ),
        })
    }
}
impl From<(HypergraphRef, IntervalGraph)> for FrontierSplitIterator {
    fn from((trav, interval): (HypergraphRef, IntervalGraph)) -> Self {
        let leaves = interval.states.leaves.iter().cloned().rev();
        FrontierSplitIterator {
            frontier: FrontierIterator {
                frontier: LinkedHashSet::from_iter(leaves),
                interval,
            },
            splits: Default::default(),
            trav,
        }
    }
}
#[derive(Debug)]
pub struct JoinContext {
    pub trav: HypergraphRef,
    pub interval: IntervalGraph,
}
#[derive(Debug)]
pub struct LockedJoinContext<'a> {
    pub trav: <HypergraphRef as HasGraphMut>::GuardMut<'a>,
    pub interval: &'a IntervalGraph,
    pub splits: &'a SplitMap,
}
impl JoinContext {
    pub fn join_subgraph(self) -> Child {
        FrontierSplitIterator::from((self.trav, self.interval))
            .find_map(|joined| joined)
            .unwrap()
    }
}

impl HasGraph for JoinContext {
    type Kind = TravKind<Hypergraph>;
    type Guard<'g>
        = <HypergraphRef as HasGraph>::Guard<'g>
    where
        Self: 'g;
    fn graph(&self) -> Self::Guard<'_> {
        self.trav.graph()
    }
}
impl HasGraphMut for JoinContext {
    type GuardMut<'g>
        = <HypergraphRef as HasGraphMut>::GuardMut<'g>
    where
        Self: 'g;
    fn graph_mut(&mut self) -> Self::GuardMut<'_> {
        self.trav.graph_mut()
    }
}
