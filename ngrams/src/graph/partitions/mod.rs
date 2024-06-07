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
use seqraph::{
    vertex::VertexIndex,
    HashSet,
};

use crate::graph::partitions::container::{
    PartitionCell,
    PartitionContainer,
};
use crate::graph::traversal::{
    BottomUp,
    TopDown,
    TraversalPolicy,
};
use crate::graph::vocabulary::{
    IndexVocab,
    VertexCtx,
};
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
pub struct PartitionsCtx<'a>
{
    #[deref]
    #[deref_mut]
    ctx: &'a mut LabellingCtx,
}

impl<'a> PartitionsCtx<'a>
{
    // find largest labelled children
    fn child_tree(
        &mut self,
        entry: &VertexCtx,
    ) -> Vec<(usize, Child)>
    {
        let mut queue: VecDeque<_> =
            TopDown::next_nodes(&entry).into_iter().collect();
        let mut tree: Vec<_> = Default::default();

        let mut visited: HashSet<_> = Default::default();
        while let Some((off, node)) = queue.pop_front()
        {
            if visited.contains(&(off, node))
            {
                continue;
            }
            visited.insert((off, node));
            if self.labels.contains(&node.index)
            {
                tree.push((off, node));
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
        &mut self,
        entry: &VertexCtx,
    ) -> PartitionContainer
    {
        let children = self.child_tree(entry);
        children.iter().tuple_windows().for_each(|((prev,_), (pos, _))|
            assert!(prev < pos, "{} < {}", prev, pos,)
        );
        PartitionContainer::from_child_list(children)
    }
    fn on_node(
        &mut self,
        vertex: &VertexCtx,
    ) -> Vec<VertexIndex>
    {
        let tree = self.child_tree(vertex);
        let container = self.partition_container(vertex);
        //println!("{:#?}", container);
        for line in container
        {
            for cell in line
            {
                let (t, s) = match cell
                {
                    PartitionCell::GapSize(s) => ("gp", s.get()),
                    PartitionCell::ChildIndex(c) => ("ch", c.width()),
                };
                print!("{}({})", t, s);
            }
            println!();
            //println!("{:#?}", line)
            //self.labels.insert(c);
        }
        vec![]
    }
    pub fn partitions_pass(
        &mut self,
        vocab: &Vocabulary,
    )
    {
        let mut queue: VecDeque<VertexIndex> = TopDown::starting_nodes(&vocab);
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
                    next_layer.extend(self.on_node(&vocab.get(&node).unwrap()));
                    visited.insert(node);
                }
            }
            queue.extend(next_layer)
        }
    }
}
