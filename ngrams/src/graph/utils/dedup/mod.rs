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

// - roots are not labelled in the beginning
// - all nodes with multiple occurences are labelled
// - there are at least two roots
// Todo:
// - visit each root child only once (use visited set) 
// - do not expand labelled or covered nodes (use child cover)
// - detect nodes that have been visited in other roots
#[derive(Debug, Default)]
pub struct DedupRoot {
    pub cover: ChildCover,
    pub visited: HashSet<VertexKey>,
}
#[derive(Debug)]
pub struct ChildDedupPass<'a> {
    pub ctx: &'a mut LabellingCtx,
    pub roots: HashMap<VertexKey, DedupRoot>,
}

impl<'a> ChildDedupPass<'a> {
    pub fn new(ctx: &'a mut LabellingCtx, roots: impl IntoIterator<Item=VertexKey>) -> Self {
        let roots: Vec<_> = roots.into_iter().collect();
        Self {
            ctx,
            roots: roots.iter().map(|root| (*root, DedupRoot::default())).collect(),
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
            self.roots.iter().flat_map(|(root, tree)| {
                self.ctx.labels.insert(*root);
                TopDown::next_nodes(&self.ctx.vocab.expect_vertex(root))
                    .into_iter()
                    .map(|(p, n)| (*root, p, n))
            })
        )
    }
    fn node_condition(&mut self, (root, _, node): Self::Node) -> bool {
        self.roots.get_mut(&root).unwrap().visited.insert(node.vertex_key())
    }
    fn on_node(&mut self, node: &Self::Node) -> Option<Vec<Self::NextNode>> {
        let &(root, off, node) = node;
        let entry = self.ctx.vocab.get_vertex(&node).unwrap();
        let (this_tree, other_trees): (Vec<_>, Vec<_>) = self.roots
                .iter_mut()
                .partition(|(k, _)| **k == root);

        let tree = this_tree.into_iter().next().unwrap().1;
        let other_trees = other_trees.into_iter().map(|(k, v)| v).collect_vec();

        // check if covered
        let next = (node.vertex_key() == root).then(|| {
            tree.cover.insert(off, node);
            Some(())
        })
        .unwrap_or_else(||
            (!tree.cover.any_covers(off, &node))
                .then_some(())
                .and_then(|_|
                    (
                        !other_trees.iter().any(|r| r.visited.contains(&node))
                    )
                    .then_some(())
                    .or_else(|| {
                        self.ctx.labels.insert(node.vertex_key());
                        tree.cover.insert(off, node);
                        None
                    })
                )
                .and_then(|_| {
                    (!self.ctx.labels.contains(&node))
                        .then_some(())
                        .or_else(|| {
                            tree.cover.insert(off, node);
                            None
                        })
                }) 
        );
        //self.visited.insert(node.vertex_key());
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