use std::{
    fmt::Debug,
    collections::VecDeque,
    hash::Hash,
};

use itertools::Itertools;

use crate::graph::vocabulary::{
    entry::VertexCtx,
    NGramId,
    Vocabulary,
};
use seqraph::graph::vertex::{
    child::Child,
    key::VertexKey,
    wide::Wide,
    VertexIndex,
};

use super::{queue::Queue, visited::{VisitTracking, VisitorCollection}};
pub trait PassNode: Eq + PartialEq + Debug + Clone + Hash {}
impl<N: Eq + PartialEq + Debug + Clone + Hash> PassNode for N {}

pub trait TraversalPass : Sized {
    type Node: PassNode + Copy;
    type NextNode: PassNode + Into<Self::Node>;
    type Queue: Queue<Self>;
    fn start_queue(&mut self) -> Self::Queue;
    fn on_node(&mut self, node: &Self::Node) -> Option<Vec<Self::NextNode>>;
    fn node_condition(&mut self, node: Self::Node) -> bool {
        true
    }
    fn begin_run(&mut self) {}
    fn finish_run(&mut self) {}
    fn run(&mut self) {
        self.begin_run();
        let mut queue = self.start_queue();

        while !queue.is_empty()
        {
            while let Some(node) = queue.pop_front()
            {
                let node = node.into();
                if let Some(next) = self.node_condition(node)
                    .then(|| self.on_node(&node))
                    .flatten()
                {
                    queue.extend_layer(next);
                }
            }
            queue.finish_layer()
        }
        self.finish_run()
    }
}
