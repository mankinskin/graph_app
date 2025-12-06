use context_trace::{
    graph::{
        vertex::{
            token::Token,
            data::{
                VertexData,
                VertexDataBuilder,
            },
            has_vertex_index::{
                HasVertexIndex,
                ToToken,
            },
            key::VertexKey,
            pattern::Pattern,
            wide::Wide,
            VertexIndex,
            ChildPatterns,
        },
        Hypergraph,
    },
    HashMap,
    HashSet,
    VertexSet,
};
use itertools::Itertools;
use pretty_assertions::assert_matches;
use range_ext::intersect::Intersect;
use std::{
    cmp::{
        Ordering,
        Reverse,
    },
    collections::VecDeque,
    fmt::{
        Display,
        Formatter,
    },
    num::NonZeroUsize,
    ops::Range,
};
use tokio_util::sync::CancellationToken;

use derive_more::{
    Deref,
    DerefMut,
    IntoIterator,
};
use derive_new::new;

use crate::graph::{
    labelling::LabellingCtx,
    partitions::{
        NodePartitionCtx,
        PartitionsCtx,
    },
    traversal::{
        direction::{
            TopDown,
            TraversalDirection,
        },
        pass::{
            RunResult,
            TraversalPass,
        },
        queue::{
            LayeredQueue,
            Queue,
        },
        visited::VisitTracking,
    },
    vocabulary::{
        entry::{
            HasVertexEntries,
            VertexCtx,
            VocabEntry,
        },
        NGramId,
        ProcessStatus,
        Vocabulary,
    },
};

use super::ChildCover;

#[derive(Debug)]
pub struct ParentCoverPass<'a> {
    pub ctx: &'a LabellingCtx,
    pub root: VertexKey,
    pub cover: ChildCover,
}
impl<'a> ParentCoverPass<'a> {
    pub fn new(
        ctx: &'a LabellingCtx,
        root: VertexKey,
    ) -> Self {
        Self {
            ctx,
            root,
            cover: Default::default(),
        }
    }
}
impl TraversalPass for ParentCoverPass<'_> {
    type Node = (usize, NGramId);
    type NextNode = (usize, NGramId);
    type Queue = LayeredQueue<Self>;
    fn ctx(&self) -> &LabellingCtx {
        self.ctx
    }
    fn start_queue(&mut self) -> RunResult<Self::Queue> {
        Ok(Self::Queue::from_iter(TopDown::next_nodes(
            &self.ctx.vocab().expect_vertex(&self.root),
        )))
    }
    fn on_node(
        &mut self,
        node: &Self::Node,
    ) -> RunResult<Option<Vec<Self::NextNode>>> {
        let &(off, node) = node;
        // check if covered
        Ok(if self.cover.any_covers(off, node) {
            None
        } else if self.ctx.labels().contains(&node) {
            self.cover.insert(off, node);
            None
        } else {
            let ne = self.ctx.vocab().get_vertex(&node).unwrap();
            Some(
                TopDown::next_nodes(&ne)
                    .into_iter()
                    .map(|(o, c)| (o + off, c))
                    .collect(),
            )
        })
    }

    fn run(&mut self) -> RunResult<()> {
        self.begin_run();
        let mut queue = self.start_queue()?;

        while !queue.is_empty() {
            while let Some(node) = queue.pop_front() {
                self.ctx().check_cancelled()?;
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
