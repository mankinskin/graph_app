use std::{
    collections::VecDeque,
    fmt::Debug,
    hash::Hash,
};

use itertools::Itertools;
use tokio_util::sync::CancellationToken;

use crate::graph::{
    labelling::LabellingCtx,
    vocabulary::{
        entry::VertexCtx,
        NGramId,
        Vocabulary,
    },
};
use context_trace::graph::vertex::{
    token::Token,
    wide::Wide,
    VertexIndex,
};

use super::{
    queue::Queue,
    visited::{
        VisitTracking,
        VisitorCollection,
    },
};
pub trait PassNode: Eq + PartialEq + Debug + Clone + Hash {}
impl<N: Eq + PartialEq + Debug + Clone + Hash> PassNode for N {}
pub enum CancelReason {
    Cancelled,
    Error,
}
#[must_use]
pub type RunResult<T> = Result<T, CancelReason>;
pub trait TraversalPass: Sized {
    type Node: PassNode + Copy;
    type NextNode: PassNode + Into<Self::Node>;
    type Queue: Queue<Self>;
    fn start_queue(&mut self) -> RunResult<Self::Queue>;
    fn on_node(
        &mut self,
        node: &Self::Node,
    ) -> RunResult<Option<Vec<Self::NextNode>>>;
    fn ctx(&self) -> &LabellingCtx;
    fn node_condition(
        &mut self,
        node: Self::Node,
    ) -> bool {
        true
    }
    fn begin_run(&mut self) {}
    fn finish_run(&mut self) -> RunResult<()> {
        Ok(())
    }
    fn run(&mut self) -> RunResult<()> {
        self.begin_run();
        let mut queue = self.start_queue()?;

        while !queue.is_empty() {
            while let Some(node) = queue.pop_front() {
                self.ctx().check_cancelled()?;
                let node = node.into();
                if self.node_condition(node) {
                    if let Some(next) = self.on_node(&node)? {
                        queue.extend_layer(next);
                    }
                }
            }
            queue.finish_layer()
        }
        self.finish_run()
    }
}
