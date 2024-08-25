mod container;

use std::collections::VecDeque;
use derive_more::{
    Deref,
    DerefMut

    ,
};
use derive_new::new;
use itertools::Itertools;

use seqraph::graph::vertex::child::Child;
use seqraph::graph::vertex::{VertexIndex, key::VertexKey};
use seqraph::graph::vertex::wide::Wide;
use seqraph::{HashMap, HashSet};
use seqraph::graph::Hypergraph;
use seqraph::graph::vertex::has_vertex_index::{HasVertexIndex, ToChild};
use seqraph::graph::vertex::data::VertexData;
use crate::graph::partitions::container::PartitionContainer;
use crate::graph::traversal::{
    TopDown,
    TraversalPolicy,
};
use crate::graph::vocabulary::{entry::{
    HasVertexEntries,
    VertexCtx,
}, ProcessStatus};
use crate::graph::labelling::LabellingCtx;

// - run top down (smaller nodes to label need to be found)
// - for each node x:
//  - run top down to find all largest labelled children
//  - arrange labelled nodes in most compact list of lists of positioned children:
//    [(p, [(x, v)])]
//  - find all nodes describing the gaps (by querying a larger node)
//  - label all gaps
#[derive(Debug, Deref, new)]
pub struct NodePartitionCtx<'a, 'b>
{
    root: Child,
    #[deref]
    ctx: &'a PartitionsCtx<'b>,
}

#[derive(Debug, Deref, DerefMut)]
pub struct PartitionsCtx<'b>
{
    #[deref]
    #[deref_mut]
    ctx: &'b mut LabellingCtx,
    graph: Hypergraph,
}

impl<'b> PartitionsCtx<'b>
{
    pub fn new(
        ctx: &'b mut LabellingCtx,
    ) -> Self {
        Self {
            ctx,
            graph: Default::default(),
        }
    }
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
                let ne = entry.vocab.get_vertex(&node.index).unwrap();
                queue.extend(
                    TopDown::next_nodes(&ne)
                        .into_iter()
                        .map(|(o, c)| (o + off, c)),
                )
            }
        }
        tree
    }
    fn on_node(
        &mut self,
        node: &VertexIndex,
    ) -> Vec<VertexIndex>
    {
        let entry = self.vocab.get_vertex(node).unwrap();
        //println!("{}", entry.ngram);

        // find all largest children
        let tree = self.child_tree(&entry);

        // build container with gaps
        //let next = tree.iter().map(|(_, c)| c.vertex_index()).collect();
        let ctx = NodePartitionCtx::new(entry.data.to_child(), self);
        let container = PartitionContainer::from_child_list(&ctx, tree);
        //println!("{:#?}", container);
        //print!("{}", container);

        let pids: Vec<_> = std::iter::repeat_n((), container.len()).map(|_| self.graph.next_pattern_id()).collect();
        assert!(self.graph.contains_vertex(node));
        let err = format!("Node not yet created {} in: {:#?}", node, &self.graph);
        let data = self.graph.get_vertex_data_mut(node).expect(&err);

        // set children
        data.children = pids.into_iter().zip(container.clone()).collect();

        // set parents for children
        let child_locations = data.all_localized_children_iter().into_iter()
            .map(|(l, c)| (l, *c))
            .collect_vec();
        assert_eq!(
            child_locations.iter().map(|(_, c)| c.vertex_index()).sorted().collect_vec(),
            container.iter().flatten().map(HasVertexIndex::vertex_index).sorted().collect_vec(),
        );
        for (loc, vi) in child_locations.into_iter() {
            if let Ok((_, v)) = self.graph.get_vertex_mut(vi) {
                v.add_parent(loc);
            } else {
                let entry = self.vocab.get_vertex(&vi.vertex_index()).unwrap();
                let mut data = VertexData::new(
                        vi.vertex_index(),
                        entry.data.width(),
                        None,
                    );
                data.add_parent(loc);
                self.graph.insert_vertex(data);
            }
        }
        // return next node indices
        let next: Vec<VertexIndex> = container.into_iter().flatten()
            .filter(|c|
                c.width() > 1
            )
            .map(|c| c.vertex_index())
            .collect();
        assert!(
            next.iter()
                .all(|v| self.graph.contains_vertex(v))
        );
        next
    }
    pub fn partitions_pass(
        &mut self,
    )
    {
        println!("Partition Pass");
        let mut queue: VecDeque<VertexIndex> = TopDown::starting_nodes(&self.vocab);
        //let mut n = 0;
        for vi in queue.iter() {
            let entry = self.vocab.get_vertex(vi).unwrap();
            self.graph.insert_vertex(
                VertexData::new(
                    *vi,
                    entry.data.width(),
                    None,
                )
            );
        }
        while !queue.is_empty()
        {
            //println!("{}", n);
            //n += 1;
            let mut visited: HashSet<_> = Default::default();
            let mut next_layer: Vec<_> = Default::default();
            while let Some(node) = queue.pop_front()
            {
                if !visited.contains(&node) && self.labels.contains(&node) &&
                    !self.vocab.leaves.contains(&node)
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
