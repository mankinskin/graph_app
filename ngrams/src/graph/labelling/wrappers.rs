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
    labelling::LabellingCtx,
    traversal::{
        BottomUp,
        TopDown,
        TraversalPolicy,
    },
    vocabulary::{
        entry::VertexCtx,
        NGramId,
        ProcessStatus,
    },
    HasVertexEntries,
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
    pub fn labelled_child_ranges(
        &self,
        entry: &VertexCtx,
    ) -> HashSet<Range<usize>>
    {
        let mut queue: VecDeque<_> =
            TopDown::next_nodes(entry).into_iter().collect();
        let mut ranges: HashSet<Range<_>> = HashSet::default();
        while !queue.is_empty()
        {
            let mut next_layer: Vec<_> = Default::default();
            while let Some((off, node)) = queue.pop_front()
            {
                if !ranges
                    .iter()
                    .any(|r| r.start <= off && off + node.width() <= r.end)
                {
                    if self.labels.contains(&node.vertex_key())
                    {
                        ranges.insert(off..off + node.width());
                    }
                    else
                    {
                        let node_entry = entry.vocab.get_vertex(&node).unwrap();
                        next_layer.extend(
                            TopDown::next_nodes(&node_entry)
                                .into_iter()
                                .map(|(o, c)| (o + off, c)),
                        );
                    }
                }
            }
            queue.extend(next_layer)
        }
        ranges
    }
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
            let ranges = self.labelled_child_ranges(&entry);
            if ranges
                .iter()
                .cartesian_product(&ranges)
                .any(|(l, r)| l != r && l.does_intersect(r))
            {
                let key = entry.data.vertex_key();
                self.labels.insert(key);
            }
        }

        // label node if it contains overlaps
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
