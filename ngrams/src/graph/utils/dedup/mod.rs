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
        pass::TraversalPass, queue::{LayeredQueue, Queue}, visited::Visited,
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

use super::cover::ChildCover;

#[derive(Debug)]
pub struct ChildDedupPass<'a> {
    pub ctx: &'a LabellingCtx,
    pub covers: HashMap<VertexKey, ChildCover>,
}

impl<'a> ChildDedupPass<'a> {
    pub fn new(ctx: &'a LabellingCtx, roots: impl IntoIterator<Item=VertexKey>) -> Self {
        Self {
            ctx,
            covers: roots.into_iter().map(|root| (root, ChildCover::default())).collect(),
        }
    }
}

impl TraversalPass for ChildDedupPass<'_> {
    type Node = (VertexKey, usize, NGramId);
    type NextNode = (VertexKey, usize, NGramId);
    type Queue = LayeredQueue<Self>;

    fn start_queue(&mut self) -> Self::Queue {
        Self::Queue::from_iter(
            //TopDown::next_nodes(&self.ctx.vocab.expect_vertex(&self.root))
            self.covers.iter().flat_map(|(key, tree)| 
                TopDown::next_nodes(&self.ctx.vocab.expect_vertex(key))
                    .into_iter()
                    .map(|(p, n)| (*key, p, n))
            )
        )
    }
    fn on_node(&mut self, node: &Self::Node) -> Option<Vec<Self::NextNode>> {
        let &(root, off, node) = node;
        let cover = self.covers.get_mut(&root).unwrap();
        // check if covered
        if cover.any_covers(off, node)
        {
            None
        }
        else if self.ctx.labels.contains(&node)
        {
            cover.insert(off, node);
            None
        }
        else
        {
            let ne = self.ctx.vocab.get_vertex(&node).unwrap();
            Some(
                TopDown::next_nodes(&ne)
                    .into_iter()
                    .map(|(o, c)| (root, o + off, c))
                    .collect()
            )
        }
    }
}