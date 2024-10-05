mod container;

use derive_more::{
    Deref,
    DerefMut,
    IntoIterator,
};
use derive_new::new;
use itertools::Itertools;
use std::collections::VecDeque;

use crate::graph::{
    labelling::LabellingCtx,
    partitions::container::{ChildTree, PartitionContainer},
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
    pub graph: Hypergraph,
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

    fn on_node(
        &mut self,
        node: &NGramId,
    ) -> Vec<NGramId>
    {
        let entry = self.vocab.get_vertex(node).unwrap();
        let container = PartitionContainer::from_entry(self, &entry);
        
        let pids: Vec<_> = std::iter::repeat_n((), container.len())
            .map(|_| PatternId::default())
            .collect();

        /////
        assert!(self.graph.contains_vertex(node.vertex_key()));
        /////

        let err = format!(
            "Node not yet created {} in: {:#?}",
            node.vertex_key(),
            &self.graph,
        );
        let parent_data = self.graph.get_vertex_mut(node.vertex_key()).expect(&err);

        /////
        assert!(
            match parent_data.width() {
                0 => panic!("Invalid width of zero."),
                2 => pids.len() == 1,
                1 => pids.is_empty(),
                _ => !pids.is_empty(),
            }
        );
        /////

        // child patterns with indices in containment
        parent_data.children = pids.into_iter().zip(container).collect();

        // child locations parent in self.graph, children indices in self.vocab.containment
        let child_locations = parent_data
            .all_localized_children_iter()
            .into_iter()
            .map(|(l, c)| (l, *c))
            .collect_vec();

        /////
        assert_eq!(
            child_locations
                .iter()
                .map(|(_, c)| c.vertex_index())
                .sorted()
                .collect_vec(),
            parent_data.children
                .iter()
                .flat_map(|(_, p)| p)
                .map(|c| c.vertex_index())
                .sorted()
                .collect_vec(),
        );
        /////


        // create child nodes in self.graph
        // set child parents and translate child indices to self.graph
        for (loc, vi) in child_locations.iter().copied()
        {
            let key = self.vocab.containment.expect_key_for_index(vi);
            let out_index = if let Ok(v) = self.graph.get_vertex_mut(key)
            {
                v.add_parent(loc);
                v.vertex_index()
            }
            else
            {
                let mut builder = VertexDataBuilder::default();
                builder.width(vi.width());
                builder.key(key);
                let mut data = self.graph.finish_vertex_builder(builder);
                assert!(data.key == key);
                data.add_parent(loc);

                // translate containment index to output index
                let out = if vi.width() > 1 {
                    self.graph.insert_vertex_data(data)
                } else {
                    self.graph.insert_token_data(*self.vocab.containment.expect_token_by_key(&key), data)
                }.vertex_index();

                if !self.ctx.labels.contains(&key) {
                    self.ctx.labels.insert(key);
                    // TODO: Rerun frequency pass for subgraph of key
                }
                out
            };
            self.graph.expect_child_mut_at(loc).index = out_index;
        }
        let parent_data = self.graph.get_vertex_mut(node.vertex_key()).expect(&err);
        child_locations
            .clone()
            .into_iter()
            .flat_map(|(_, p)| p)
            .filter(|c| c.width() > 1)
            .map(|c| {
                let entry = self.vocab.get_vertex(&c).unwrap();
                let key = entry.data.vertex_key();
                assert!(
                    self.ctx.labels.contains(&key),
                );
                assert!(
                    self.graph.contains_vertex(key),
                    "{:#?}", entry.entry,
                );
                NGramId::new(
                    key,
                    c.width(),
                )
            })
            .collect()
    }
    pub fn partitions_pass(&mut self)
    {
        println!("Partition Pass");
        let mut queue: VecDeque<_> = TopDown::starting_nodes(&self.vocab);
        for vk in queue.iter()
        {
            let data = self.vocab.containment.expect_vertex(vk.vertex_key());
            let mut builder = VertexDataBuilder::default();
            builder.width(data.width());
            builder.key(**vk);
            self.graph.insert_vertex_builder(builder);
        }

        while !queue.is_empty()
        {
            let mut visited: HashSet<_> = Default::default();
            let mut next_layer: Vec<_> = Default::default();
            while let Some(node) = queue.pop_front()
            {
                if (!visited.contains(&node)
                    && self.labels.contains(&node))
                    || self.vocab.leaves.contains(&node)
                {
                    next_layer.extend(self.on_node(&node));
                    visited.insert(node);
                }
            }
            queue.extend(next_layer)
        }
        self.vocab.roots.iter().for_each(|key| {
            let _ = self.graph.vertex_key_string(key);
        });
        self.status = ProcessStatus::Partitions;
        println!("{:#?}", &self.graph);
    }
}
