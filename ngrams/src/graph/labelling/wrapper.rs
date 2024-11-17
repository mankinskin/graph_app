use std::{
    collections::VecDeque,
    ops::Range,
};

use derive_more::{
    Deref,
    DerefMut,
    From,
};
use derive_new::new;
use itertools::Itertools;
use range_ext::intersect::Intersect;

use crate::graph::{
    labelling::LabellingCtx, traversal::{
        direction::{
            BottomUp,
            TopDown,
            TraversalDirection,
        },
        pass::TraversalPass, queue::{LayeredQueue, Queue}, visited::VisitTracking,
    }, utils::cover::ChildCover, vocabulary::{
        entry::VertexCtx,
        NGramId,
        ProcessStatus, Vocabulary,
    }, HasVertexEntries
};
use seqraph::{
    graph::vertex::{
        has_vertex_index::HasVertexIndex,
        has_vertex_key::HasVertexKey,
        key::VertexKey,
        wide::Wide,
        VertexIndex,
    },
    HashSet,
};

#[derive(Debug, Deref, new, DerefMut)]
pub struct WrapperCtx<'b>
{
    #[deref]
    #[deref_mut]
    ctx: &'b mut LabellingCtx,
    #[new(default)]
    visited_mut: <Self as VisitTracking>::Collection,
}
// - run bottom up (all smaller nodes need to be fully labelled)
// - for each node x:
//  - run top down to find the largest frequent children to cover whole range
//  - label node x if there are multiple overlapping labelled child nodes

impl VisitTracking for WrapperCtx<'_>
{
    type Collection = HashSet<<Self as TraversalPass>::Node>;
    fn visited_mut(&mut self) -> &mut <Self as VisitTracking>::Collection {
        &mut self.visited_mut
    }
}
impl TraversalPass for WrapperCtx<'_>
{
    type Node = VertexKey;
    type NextNode = VertexKey;
    type Queue = LayeredQueue<Self>;
    fn start_queue(&mut self) -> Self::Queue {
        BottomUp::starting_nodes(&self.vocab).into_iter()
            .map(|ng| ng.key).collect()
    }
    fn on_node(
        &mut self,
        node: &Self::Node,
    ) -> Option<Vec<Self::NextNode>>
    {
        let entry = self.vocab.get_vertex(node).unwrap();
        let next = BottomUp::next_nodes(&entry)
            .iter()
            .map(HasVertexKey::vertex_key)
            .collect();

        if !self.labels.contains(node)
        {
            let tree = ChildCover::from_key(self.ctx, entry.vertex_key());
            if tree.any_intersect()
            {
                let key = entry.data.vertex_key();
                // label node if it contains overlaps
                self.labels.insert(key);
            }
        }

        Some(next)
    }
    fn begin_run(&mut self) {
        println!("Wrapper Pass");
    }
    fn finish_run(&mut self) {
        self.status = ProcessStatus::Wrappers;
    }
}
