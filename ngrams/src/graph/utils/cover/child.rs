use itertools::Itertools;
use pretty_assertions::assert_matches;
use range_ext::intersect::Intersect;
use seqraph::{
    graph::{
        getters::vertex::VertexSet,
        vertex::{
            child::Child, data::{
                VertexData,
                VertexDataBuilder,
            }, has_vertex_index::{
                HasVertexIndex,
                ToChild,
            }, has_vertex_key::HasVertexKey, key::VertexKey, wide::Wide, VertexIndex
        },
        Hypergraph,
    },
    HashMap,
    HashSet,
};
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

use derive_new::new;
use derive_more::{
    Deref,
    DerefMut,
    IntoIterator,
};

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
        pass::TraversalPass, queue::{LayeredQueue, Queue}, visited::VisitTracking,
    },
    vocabulary::{
        entry::{
            HasVertexEntries,
            VertexCtx,
            VocabEntry,
        },
        NGramId,
        ProcessStatus, Vocabulary,
    },
};

use super::ChildCover;

#[derive(Debug)]
pub struct ChildCoverPass<'a> {
    pub ctx: &'a LabellingCtx,
    pub root: VertexKey,
    pub cover: ChildCover,
}
impl<'a> ChildCoverPass<'a> {
    pub fn new(ctx: &'a LabellingCtx, root: VertexKey) -> Self {
        Self {
            ctx,
            root,
            cover: Default::default(),
        }
    }
}
impl TraversalPass for ChildCoverPass<'_> {
    type Node = (usize, NGramId);
    type NextNode = (usize, NGramId);
    type Queue = LayeredQueue<Self>;
    fn start_queue(&mut self) -> Self::Queue {
        Self::Queue::from_iter(
            TopDown::next_nodes(&self.ctx.vocab.expect_vertex(&self.root))
        )
    }
    fn on_node(&mut self, node: &Self::Node) -> Option<Vec<Self::NextNode>> {
        let &(off, node) = node;
        // check if covered
        if self.cover.any_covers(off, node)
        {
            None
        }
        else if self.ctx.labels.contains(&node)
        {
            self.cover.insert(off, node);
            None
        }
        else
        {
            let ne = self.ctx.vocab.get_vertex(&node).unwrap();
            Some(
                TopDown::next_nodes(&ne)
                    .into_iter()
                    .map(|(o, c)| (o + off, c))
                    .collect()
            )
        }
    }
}
