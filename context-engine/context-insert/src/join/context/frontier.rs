use super::node::context::NodeJoinCtx;
use itertools::Itertools;
use linked_hash_set::LinkedHashSet;

use crate::{
    interval::IntervalGraph,
    split::{
        SplitMap,
        cache::position::PosKey,
    },
};
use context_trace::graph::{
    HypergraphRef,
    vertex::{
        child::Child,
        wide::Wide,
    },
};

pub struct FrontierIterator {
    pub(crate) frontier: LinkedHashSet<PosKey>,
    pub(crate) interval: IntervalGraph,
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
    pub(crate) frontier: FrontierIterator,
    pub(crate) splits: SplitMap,
    pub(crate) trav: HypergraphRef,
}

impl FrontierSplitIterator {
    fn node<'a>(
        &'a mut self,
        index: Child,
    ) -> NodeJoinCtx<'a> {
        NodeJoinCtx::new(index, self)
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
