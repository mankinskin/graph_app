mod container;

use derive_more::{
    Deref,
    DerefMut,
};
use derive_new::new;
use itertools::Itertools;
use std::collections::VecDeque;

use crate::graph::{
    labelling::LabellingCtx,
    partitions::container::PartitionContainer,
    traversal::{
        TopDown,
        TraversalPolicy,
    },
    vocabulary::{
        entry::{
            HasVertexEntries,
            VertexCtx,
        },
        NGramId,
        ProcessStatus,
    },
};
use seqraph::{
    graph::{
        getters::vertex::VertexSet,
        vertex::{
            child::Child,
            data::{
                VertexData,
                VertexDataBuilder,
            },
            has_vertex_index::{
                HasVertexIndex,
                ToChild,
            },
            has_vertex_key::HasVertexKey,
            key::VertexKey,
            pattern::id::PatternId,
            wide::Wide,
            VertexIndex,
        },
        Hypergraph,
    },
    HashMap,
    HashSet,
};

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
    root: NGramId,
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
    pub fn new(ctx: &'b mut LabellingCtx) -> Self
    {
        Self {
            ctx,
            graph: Default::default(),
        }
    }
    // find largest labelled children
    fn child_tree(
        &self,
        entry: &VertexCtx,
    ) -> HashMap<usize, NGramId>
    {
        let mut queue: VecDeque<_> =
            TopDown::next_nodes(entry).into_iter().collect();
        let mut tree: HashMap<usize, NGramId> = Default::default();

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
            if self.labels.contains(&node)
            {
                tree.insert(off, node);
            }
            else
            {
                let ne = entry.vocab.get_vertex(&node).unwrap();
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
        node: &NGramId,
    ) -> Vec<NGramId>
    {
        let entry = self.vocab.get_vertex(node).unwrap();
        //println!("{}", entry.ngram);

        // find all largest children
        let tree = self.child_tree(&entry);

        // build container with gaps
        //let next = tree.iter().map(|(_, c)| c.vertex_index()).collect();
        let ctx = NodePartitionCtx::new(
            NGramId::new(entry.data.vertex_key(), entry.data.width()),
            self,
        );
        let container = PartitionContainer::from_child_list(&ctx, tree);
        //println!("{:#?}", container);
        //print!("{}", container);

        let pids: Vec<_> = std::iter::repeat_n((), container.len())
            .map(|_| PatternId::default())
            .collect();
        assert!(self.graph.contains_vertex(node.vertex_key()));
        let err = format!(
            "Node not yet created {} in: {:#?}",
            node.vertex_key(),
            &self.graph
        );
        let data = self.graph.get_vertex_mut(node.vertex_key()).expect(&err);

        // set children
        data.children = pids.into_iter().zip(container.clone()).collect();

        // set parents for children
        let child_locations = data
            .all_localized_children_iter()
            .into_iter()
            .map(|(l, c)| (l, *c))
            .collect_vec();
        assert_eq!(
            child_locations
                .iter()
                .map(|(_, c)| c.vertex_index())
                .sorted()
                .collect_vec(),
            container
                .iter()
                .flatten()
                .map(HasVertexIndex::vertex_index)
                .sorted()
                .collect_vec(),
        );
        for (loc, vi) in child_locations.into_iter()
        {
            if let Ok(v) = self.graph.get_vertex_mut(vi)
            {
                v.add_parent(loc);
            }
            else
            {
                let entry = self.vocab.get_vertex(&vi).unwrap();
                let mut data =
                    VertexData::new(vi.vertex_index(), entry.data.width());
                data.add_parent(loc);
                self.graph.insert_vertex_data(data);
            }
        }
        // return next node indices
        let next: Vec<_> = container
            .into_iter()
            .flatten()
            .filter(|c| c.width() > 1)
            .map(|c| {
                NGramId::new(
                    self.vocab.containment.expect_key_for_index(c),
                    c.width(),
                )
            })
            .collect();
        assert!(next
            .iter()
            .all(|v| self.graph.contains_vertex(v.vertex_key())));
        next
    }
    pub fn partitions_pass(&mut self)
    {
        println!("Partition Pass");
        let mut queue: VecDeque<_> = TopDown::starting_nodes(&self.vocab);
        //let mut n = 0;
        for vk in queue.iter()
        {
            let entry = self.vocab.get_vertex(vk).unwrap();
            let mut builder = VertexDataBuilder::default();
            builder.width(entry.data.width());
            self.graph.insert_vertex_builder(builder);
        }
        while !queue.is_empty()
        {
            //println!("{}", n);
            //n += 1;
            let mut visited: HashSet<_> = Default::default();
            let mut next_layer: Vec<_> = Default::default();
            while let Some(node) = queue.pop_front()
            {
                if !visited.contains(&node)
                    && self.labels.contains(&node)
                    && !self.vocab.leaves.contains(&node)
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
