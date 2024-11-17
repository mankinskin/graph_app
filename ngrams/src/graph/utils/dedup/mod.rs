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
    }, collections::VecDeque, fmt::{
        Display,
        Formatter,
    }, hash::Hash, num::NonZeroUsize, ops::Range
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
        pass::TraversalPass, queue::{LayeredQueue, LinearQueue, Queue}, visited::VisitTracking,
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
    pub ctx: &'a mut LabellingCtx,
    pub covers: HashMap<VertexKey, ChildCover>,
    roots: Vec<VertexKey>,
    pub visited: HashSet<VertexKey>,
}

impl<'a> ChildDedupPass<'a> {
    pub fn new(ctx: &'a mut LabellingCtx, roots: impl IntoIterator<Item=VertexKey>) -> Self {
        let roots: Vec<_> = roots.into_iter().collect();
        Self {
            ctx,
            covers: roots.iter().map(|root| (*root, ChildCover::default())).collect(),
            roots,
            visited: Default::default(),
        }
    }
}
impl TraversalPass for ChildDedupPass<'_> {
    type Node = (VertexKey, usize, NGramId);
    type NextNode = (VertexKey, usize, NGramId);
    type Queue = LinearQueue<Self>;

    fn start_queue(&mut self) -> Self::Queue {
        Self::Queue::from_iter(
            //self.roots.iter()
            //    .map(|&root| (0, NGramId::new(root, self.ctx.vocab.expect_vertex(&root).width())))
            self.covers.iter().flat_map(|(root, tree)| {
                self.ctx.labels.insert(*root);
                TopDown::next_nodes(&self.ctx.vocab.expect_vertex(root))
                    .into_iter()
                    .map(|(p, n)| (*root, p, n))
            })
        )
    }
    fn on_node(&mut self, node: &Self::Node) -> Option<Vec<Self::NextNode>> {
        let &(root, off, node) = node;
        let entry = self.ctx.vocab.get_vertex(&node).unwrap();
        if entry.ngram == "ab" || entry.ngram == "ba" {
            let re = self.ctx.vocab.get_vertex(&root).unwrap();
            println!("{}({})found in root {}({})", entry.ngram, entry.vertex_key(), re.ngram, root);
        }
        let cover = self.covers.get_mut(&root).unwrap();
        // check if covered
        let next = (node.vertex_key() == root).then(|| {
            cover.insert(off, node);
            Some(())
        })
        .unwrap_or_else(||
            if cover.any_covers(off, &node)
            {
                None
            }
            else if self.visited.contains(&node)
            {
                self.ctx.labels.insert(node.vertex_key());
                cover.insert(off, node);
                None
            }
            else if self.ctx.labels.contains(&node)
            {
                cover.insert(off, node);
                None
            } else {
                Some(())
            }
        );
        self.visited.insert(node.vertex_key());
        next.map(|_| {
            let ne = self.ctx.vocab.get_vertex(&node).unwrap();
            TopDown::next_nodes(&ne)
                .into_iter()
                .map(|(o, c)| (root, o + off, c))
                .collect()
        })
    }
    //fn finish_run(&mut self) {
    //    self.roots.iter().for_each(|&root| {
    //        self.covers
    //            .get_mut(&root).unwrap()
    //            .insert(
    //                0,
    //                NGramId::new(
    //                    root,
    //                    self.ctx.vocab.expect_vertex(&root).width(),
    //                )
    //            );
    //    })
    //}
}