use std::collections::VecDeque;

use derive_more::{
    Deref,
    DerefMut,
    From,
};
use itertools::Itertools;
use range_ext::intersect::Intersect;

use seqraph::{
    vertex::wide::Wide,
    vertex::VertexIndex,
    HashSet,
};

use crate::graph::traversal::{
    BottomUp,
    TopDown,
    TraversalPolicy,
};
use crate::graph::vocabulary::entry::VertexCtx;
use crate::graph::{
    labelling::LabellingCtx,
    IndexVocab,
    Vocabulary,
};
use crate::graph::vocabulary::ProcessStatus;

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

impl<'b> WrapperCtx<'b>
{
    pub fn on_node(
        &mut self,
        node: &VertexIndex,
    ) -> Vec<VertexIndex>
    {
        let entry = self.vocab.get(node).unwrap();
        let mut queue: VecDeque<_> =
            TopDown::next_nodes(&entry).into_iter().collect();
        let mut ranges: HashSet<_> = HashSet::default();

        while !queue.is_empty()
        {
            let mut visited: HashSet<_> = Default::default();
            let mut next_layer: Vec<_> = Default::default();
            while let Some((off, node)) = queue.pop_front()
            {
                visited.insert(node.index);
                if self.labels.contains(&node.index)
                {
                    ranges.insert(off..off + node.width());
                }
                else
                {
                    let ne = entry.vocab.get(&node.index).unwrap();
                    next_layer.extend(
                        TopDown::next_nodes(&ne).into_iter().filter_map(
                            |(o, c)| {
                                (!visited.contains(&c.index))
                                    .then(|| (o + off, c))
                            },
                        ),
                    );
                }
            }
            queue.extend(next_layer)
        }
        //println!("ranges finished");
        let next = BottomUp::next_nodes(&entry);
        if ranges
            .iter()
            .cartesian_product(&ranges)
            .find(|(l, r)| l.does_intersect(*r))
            .is_some()
        {
            //println!("wrapper");
            let index = entry.data.index;
            self.labels.insert(index);
        }
        next
    }
    pub fn wrapping_pass(
        &mut self,
    )
    {
        let mut queue: VecDeque<VertexIndex> = BottomUp::starting_nodes(&self.vocab);
        while !queue.is_empty()
        {
            let mut visited: HashSet<_> = Default::default();
            let mut next_layer: Vec<_> = Default::default();
            while let Some(node) = queue.pop_front()
            {
                if !visited.contains(&node) && !self.labels.contains(&node)
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
