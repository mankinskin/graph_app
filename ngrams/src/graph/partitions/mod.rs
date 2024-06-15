mod container;

use pretty_assertions::assert_matches;
use std::cmp::{
    Ordering,
    Reverse,
};
use std::collections::VecDeque;
use std::num::NonZeroUsize;

use derive_more::{
    Deref,
    DerefMut,
    From,
    IntoIterator,
};
use itertools::Itertools;

use seqraph::vertex::child::Child;
use seqraph::vertex::wide::Wide;
use seqraph::{vertex::VertexIndex, HashSet, HashMap};

use crate::graph::partitions::container::{
    PartitionCell,
    PartitionContainer,
};
use crate::graph::traversal::{
    BottomUp,
    TopDown,
    TraversalPolicy,
};
use crate::graph::vocabulary::{entry::{
    IndexVocab,
    VertexCtx,
}, ProcessStatus};
use crate::graph::{
    labelling::LabellingCtx,
    vocabulary::Vocabulary,
};

// - run top down (smaller nodes to label need to be found)
// - for each node x:
//  - run top down to find all largest labelled children
//  - arrange labelled nodes in most compact list of lists of positioned children:
//    [(p, [(x, v)])]
//  - find all nodes describing the gaps (by querying a larger node)
//  - label all gaps

#[derive(Debug, Deref, From, DerefMut)]
pub struct PartitionsCtx<'b>
{
    #[deref]
    #[deref_mut]
    ctx: &'b mut LabellingCtx,
}

impl<'b> PartitionsCtx<'b>
{
    // find largest labelled children
    fn child_tree(
        &self,
        entry: &VertexCtx,
    ) -> HashMap<usize, Child>
    {
        let mut queue: VecDeque<_> =
            TopDown::next_nodes(&entry).into_iter().collect();
        let mut tree: HashMap<usize, Child> = Default::default();

        let mut visited: HashSet<_> = Default::default();
        while let Some((off, node)) = queue.pop_front()
        {
            if visited.contains(&(off, node))
            {
                continue;
            }
            visited.insert((off, node));
            // check if covered
            if tree.iter().any(|(&p, &c)| {
                let node_end = off + node.width();
                let probe_end = p + c.width();
                p <= off && node_end <= probe_end
            })
            {
                continue;
            }
            if self.labels.contains(&node.index)
            {
                tree.insert(off, node);
            }
            else
            {
                let ne = entry.vocab.get(&node.index).unwrap();
                queue.extend(
                    TopDown::next_nodes(&ne)
                        .into_iter()
                        .map(|(o, c)| (o + off, c)),
                )
            }
        }
        tree
    }
    fn partition_container(
        &self,
        entry: &VertexCtx,
    ) -> PartitionContainer
    {
        let children = self.child_tree(entry);
        PartitionContainer::from_child_list(children)
    }
    fn on_node(
        &mut self,
        node: &VertexIndex,
    ) -> Vec<VertexIndex>
    {
        let entry = self.vocab.get(node).unwrap();
        //println!("{}", entry.ngram);
        let tree = self.child_tree(&entry);
        let container = self.partition_container(&entry);
        //println!("{:#?}", container);
        //print!("{}", container);
        vec![]
    }
    pub fn partitions_pass(
        &mut self,
    )
    {
        println!("Partition Pass");
        let mut queue: VecDeque<VertexIndex> = TopDown::starting_nodes(&self.vocab);
        let mut n = 0;
        while !queue.is_empty()
        {
            n += 1;
            println!("{}", n);
            let mut visited: HashSet<_> = Default::default();
            let mut next_layer: Vec<_> = Default::default();
            while let Some(node) = queue.pop_front()
            {
                if !visited.contains(&node) && self.labels.contains(&node)
                {
                    next_layer.extend(self.on_node(&node));
                    visited.insert(node);
                }
            }
            queue.extend(next_layer)
        }
        self.status = ProcessStatus::Partitions;
    }
}
