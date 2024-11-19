pub mod container;
pub mod collect;

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
    partitions::container::PartitionContainer,
    traversal::{
        direction::{
            TopDown,
            TraversalDirection,
        },
        pass::TraversalPass, queue::Queue,
    },
    utils::cover::ChildCover,
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

use super::{traversal::{queue::{LayeredQueue, LinearQueue}, visited::VisitTracking}, vocabulary::Vocabulary, utils::dedup::ChildDedupPass};

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
    pub ctx: &'b mut LabellingCtx,
    visited: <Self as VisitTracking>::Collection,
    pub graph: Hypergraph,
}

impl<'b> From<&'b mut LabellingCtx> for PartitionsCtx<'b> {
    fn from(ctx: &'b mut LabellingCtx) -> Self
    {
        Self {
            ctx,
            visited: Default::default(),
            graph: Default::default(),
        }
    }
}
impl VisitTracking for PartitionsCtx<'_>
{
    type Collection = HashSet<<Self as TraversalPass>::Node>;
    fn visited_mut(&mut self) -> &mut <Self as VisitTracking>::Collection {
        &mut self.visited
    }
}
impl TraversalPass for PartitionsCtx<'_>
{
    type Node = NGramId;
    type NextNode = NGramId;
    type Queue = LinearQueue<Self>;
    fn start_queue(&mut self) -> Self::Queue {
        let queue = Self::Queue::from_iter(
            TopDown::starting_nodes(&self.vocab)
        );
        for vk in queue.iter()
        {
            let data = self.vocab.containment.expect_vertex(vk.vertex_key());
            let mut builder = VertexDataBuilder::default();
            builder.width(data.width());
            builder.key(**vk);
            self.graph.insert_vertex_builder(builder);
        }
        self.status.as_ref().inspect(|s|
            s.write().unwrap().next_pass(ProcessStatus::Wrappers, 0, 100)
        );
        queue
    }
    fn node_condition(&mut self, node: Self::Node) -> bool {
        (!self.visited_mut().contains(&node)
            && self.labels.contains(&node))
            || self.vocab.leaves.contains(&node)
            .then(|| self.visited_mut().insert(node))
            .is_some()
    }
    fn on_node(
        &mut self,
        node: &NGramId,
    ) -> Option<Vec<NGramId>>
    {
        self.status.as_ref().inspect(|s| s.write().unwrap().steps += 1);
        let container = PartitionContainer::from_ngram(self, *node);
        let entry = self.vocab.get_vertex(node).unwrap();
        
        let pids: Vec<_> = std::iter::repeat_n((), container.len())
            .map(|_| PatternId::default())
            .collect();

        let parent_data = self.graph.expect_vertex_mut(node.vertex_key());

        // child patterns with indices in containment
        parent_data.children = pids.into_iter().zip(container).collect();

        // child locations parent in self.graph, children indices in self.vocab.containment
        let child_locations = parent_data
            .all_localized_children_iter()
            .into_iter()
            .map(|(l, c)| (l, *c))
            .collect_vec();

        let unlabelled = child_locations
            .iter()
            .map(|(_, vi)| self.vocab.containment.expect_key_for_index(*vi))
            .filter(|k| !self.labels.contains(k))
            .collect_vec();

        ChildDedupPass::new(self.ctx, unlabelled).run();

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
                data.add_parent(loc);

                // translate containment index to output index
                let out = if vi.width() > 1 {
                    self.graph.insert_vertex_data(data)
                } else {
                    self.graph.insert_token_data(*self.vocab.containment.expect_token_by_key(&key), data)
                }.vertex_index();

                out
            };
            self.graph.expect_child_mut_at(loc).index = out_index;
        }
        let next = child_locations
            .clone()
            .into_iter()
            .flat_map(|(_, p)| p)
            .filter(|c| c.width() > 1)
            .map(|c|
                NGramId::new(
                    self.vocab.get_vertex(&c).unwrap().data.vertex_key(),
                    c.width(),
                )
            )
            .collect();
        //let next = vec![];
        Some(next)
    }
    fn begin_run(&mut self) {
        println!("Partition Pass");
    }

    fn finish_run(&mut self) {
        self.vocab.roots.iter().for_each(|key| {
            let _ = self.graph.vertex_key_string(key);
        });
        println!("{:#?}", &self.graph);
        self.status.as_ref().inspect(|s| s.write().unwrap().pass = ProcessStatus::Partitions);
    }
}
