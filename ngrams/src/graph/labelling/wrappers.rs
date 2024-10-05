use std::{
    collections::VecDeque,
    ops::Range,
};

use derive_more::{
    Deref,
    DerefMut,
    From,
};
use itertools::Itertools;
use range_ext::intersect::Intersect;

use crate::graph::{
    labelling::LabellingCtx, partitions::container::ChildTree, traversal::{
        BottomUp,
        TopDown,
        TraversalPolicy,
    }, vocabulary::{
        entry::VertexCtx,
        NGramId,
        ProcessStatus,
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

#[derive(Debug, Deref, From, DerefMut)]
pub struct WrapperCtx<'b>
{
    #[deref]
    #[deref_mut]
    ctx: &'b mut LabellingCtx,
}
// - run bottom up (all smaller nodes need to be fully labelled)
// - for each node x:
//  - run top down to find the largest frequent children to cover whole range
//  - label node x if there are multiple overlapping labelled child nodes

impl WrapperCtx<'_>
{
    pub fn on_node(
        &mut self,
        node: &VertexKey,
    ) -> Vec<VertexKey>
    {
        let entry = self.vocab.get_vertex(node).unwrap();
        let next = BottomUp::next_nodes(&entry)
            .iter()
            .map(HasVertexKey::vertex_key)
            .collect();

        if !self.labels.contains(node)
        {
            let tree = ChildTree::from_entry(self.ctx, &entry);
            if tree.any_intersect()
            {
                let key = entry.data.vertex_key();
                // label node if it contains overlaps
                self.labels.insert(key);
            }
        }

        next
    }
    pub fn wrapping_pass(&mut self)
    {
        println!("Wrapper Pass");
        let mut queue: VecDeque<_> = BottomUp::starting_nodes(&self.vocab)
            .iter()
            .map(HasVertexKey::vertex_key)
            .collect();
        while !queue.is_empty()
        {
            let mut visited: HashSet<_> = Default::default();
            let mut next_layer: Vec<_> = Default::default();

            while let Some(node) = queue.pop_front()
            {
                if !visited.contains(&node)
                {
                    visited.insert(node);
                    next_layer.extend(self.on_node(&node));
                }
            }
            queue.extend(next_layer)
        }
        self.status = ProcessStatus::Wrappers;
    }
}
