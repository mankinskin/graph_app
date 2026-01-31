pub mod collect;
pub mod container;

use derive_more::{
    Deref,
    DerefMut,
    IntoIterator,
};
use derive_new::new;
use itertools::Itertools;
use std::collections::VecDeque;
use tokio_util::sync::CancellationToken;

use crate::graph::{
    labelling::LabellingCtx,
    partitions::container::PartitionContainer,
    traversal::{
        direction::{
            TopDown,
            TraversalDirection,
        },
        pass::{
            RunResult,
            TraversalPass,
        },
        queue::Queue,
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
use context_trace::{
    graph::{
        vertex::{
            data::{
                VertexData,
                VertexDataBuilder,
            },
            has_vertex_index::{
                HasVertexIndex,
                ToToken,
            },
            has_vertex_key::HasVertexKey,
            location::SubLocation,
            pattern::{
                id::PatternId,
                Pattern,
            },
            token::Token,
            wide::Wide,
            ChildPatterns,
            VertexIndex,
        },
        Hypergraph,
    },
    HashMap,
    HashSet,
    VertexSet,
};

use super::{
    traversal::{
        queue::{
            LayeredQueue,
            LinearQueue,
        },
        visited::VisitTracking,
    },
    utils::dedup::ChildDedupPass,
    vocabulary::Vocabulary,
};

// - run top down (smaller nodes to label need to be found)
// - for each node x:
//  - run top down to find all largest labelled children
//  - arrange labelled nodes in most compact list of lists of positioned children:
//    [(p, [(x, v)])]
//  - find all nodes describing the gaps (by querying a larger node)
//  - label all gaps
#[derive(Debug, Deref, new)]
pub struct NodePartitionCtx<'a, 'b> {
    root: NGramId,
    #[deref]
    ctx: &'a PartitionsCtx<'b>,
}

#[derive(Debug, Deref, DerefMut)]
pub struct PartitionsCtx<'b> {
    #[deref]
    #[deref_mut]
    pub ctx: &'b mut LabellingCtx,
    visited: <Self as VisitTracking>::Collection,
    pub graph: Hypergraph,
}

impl<'b> From<&'b mut LabellingCtx> for PartitionsCtx<'b> {
    fn from(ctx: &'b mut LabellingCtx) -> Self {
        Self {
            ctx,
            visited: Default::default(),
            graph: Default::default(),
        }
    }
}
impl VisitTracking for PartitionsCtx<'_> {
    type Collection = HashSet<<Self as TraversalPass>::Node>;
    fn visited_mut(&mut self) -> &mut <Self as VisitTracking>::Collection {
        &mut self.visited
    }
}
impl TraversalPass for PartitionsCtx<'_> {
    type Node = NGramId;
    type NextNode = NGramId;
    type Queue = LinearQueue<Self>;
    fn ctx(&self) -> &LabellingCtx {
        self.ctx
    }
    fn start_queue(&mut self) -> RunResult<Self::Queue> {
        let queue =
            Self::Queue::from_iter(TopDown::starting_nodes(self.vocab()));
        for vk in queue.iter() {
            let data =
                self.vocab().containment.expect_vertex_data(vk.vertex_key());
            let builder =
                VertexDataBuilder::default().width(data.width()).key(**vk);
            self.graph.insert_vertex_builder(builder);
        }
        self.status.next_pass(
            ProcessStatus::Partitions,
            0,
            self.labels().len() + self.vocab().leaves.len(),
        );
        Ok(queue)
    }
    fn node_condition(
        &mut self,
        node: Self::Node,
    ) -> bool {
        (!self.visited_mut().contains(&node) && self.labels().contains(&node))
            || self
                .vocab()
                .leaves
                .contains(&node)
                .then(|| self.visited_mut().insert(node))
                .is_some()
    }
    fn on_node(
        &mut self,
        node: &NGramId,
    ) -> RunResult<Option<Vec<Self::NextNode>>> {
        *self.status.steps_mut() += 1;
        let container = PartitionContainer::from_ngram(self, *node);
        let entry = self.vocab().get_vertex(node).unwrap();

        let pids: Vec<_> = std::iter::repeat_n((), container.len())
            .map(|_| PatternId::default())
            .collect();

        // Build child patterns and get child locations in one mutation
        let child_locations = self
            .graph
            .with_vertex_mut(node.vertex_key(), |parent_data| {
                // child patterns with indices in containment
                *parent_data.child_patterns_mut() = pids
                    .into_iter()
                    .zip(container)
                    .map(|(pid, tokens)| (pid, Pattern::from(tokens)))
                    .collect();

                // child locations parent in self.graph, children indices in self.vocab.containment
                parent_data
                    .all_localized_children_iter()
                    .into_iter()
                    .map(|(l, c)| (l, *c))
                    .collect_vec()
            })
            .unwrap();

        let unlabelled = child_locations
            .iter()
            .map(|(_, vi)| self.vocab().containment.expect_key_for_index(*vi))
            .filter(|k| !self.labels().contains(k))
            .collect_vec();

        ChildDedupPass::new(self.ctx, unlabelled).run();

        // create child nodes in self.graph
        // set child parents and translate child indices to self.graph
        for (loc, vi) in child_locations.iter().copied() {
            let key = self.vocab().containment.expect_key_for_index(vi);
            let out_index = if self.graph.contains_vertex(key) {
                self.graph
                    .with_vertex_mut(key, |v| {
                        v.add_parent(loc);
                        v.vertex_index()
                    })
                    .unwrap()
            } else {
                let builder =
                    VertexDataBuilder::default().width(vi.width()).key(key);
                let mut data = self.graph.finish_vertex_builder(builder);
                data.add_parent(loc);

                // translate containment index to output index
                self.graph.insert_vertex_data(data).vertex_index()
            };
            // Update the child token in the parent node
            let sub_loc = SubLocation::from(loc);
            self.graph
                .with_vertex_mut(node.vertex_key(), |parent_data| {
                    parent_data.expect_child_mut_at(&sub_loc).index = out_index;
                })
                .unwrap();
        }
        let next = child_locations
            .clone()
            .into_iter()
            .flat_map(|(_, p)| p)
            .filter(|c| c.width() > 1)
            .map(|c| {
                NGramId::new(
                    self.vocab().get_vertex(&c).unwrap().data.vertex_key(),
                    c.width().0,
                )
            })
            .collect();
        //let next = vec![];
        Ok(Some(next))
    }
    fn begin_run(&mut self) {
        println!("Partition Pass");
    }

    fn finish_run(&mut self) -> RunResult<()> {
        self.vocab().roots.iter().for_each(|key| {
            let _ = self.graph.vertex_key_string(key);
        });
        //println!("{:#?}", &self.graph);
        *self.status.pass_mut() = ProcessStatus::Finished;
        Ok(())
    }
}
