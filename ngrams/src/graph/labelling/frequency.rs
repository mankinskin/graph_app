
use std::collections::VecDeque;

use derive_more::{
    Deref,
    DerefMut,
    From,
};
use derive_new::new;
use itertools::Itertools;

use crate::graph::{
    containment::TextLocation, labelling::LabellingCtx, traversal::{
        direction::{
            BottomUp,
            TopDown,
            TraversalDirection,
        }, pass::TraversalPass, queue::{Queue, SortedQueue}
    }, utils::cover::frequency::FrequencyCover, vocabulary::{
        entry::{
            HasVertexEntries,
            VertexCtx,
        },
        NGramId,
        ProcessStatus,
        Vocabulary,
    }
};
use seqraph::{
    graph::vertex::{
        child::Child,
        has_vertex_index::HasVertexIndex,
        has_vertex_key::HasVertexKey,
        key::VertexKey,
        location::child::ChildLocation,
        wide::Wide,
        VertexIndex,
    },
    HashSet,
};

#[derive(Debug, Deref, new, DerefMut)]
pub struct FrequencyCtx<'b>
{
    #[deref]
    #[deref_mut]
    pub ctx: &'b mut LabellingCtx,
}

impl TraversalPass for FrequencyCtx<'_>
{
    type Node = VertexKey;
    type NextNode = NGramId;
    type Queue = SortedQueue;
    fn start_queue(&mut self) -> Self::Queue {
        let start = TopDown::starting_nodes(&self.vocab);

        let mut queue = SortedQueue::default();
        for node in start.iter()
        {
            queue.extend_layer(
                self.on_node(node).unwrap_or_default()
            );
        }
        self.labels.extend(start.iter().map(HasVertexKey::vertex_key));
        queue
    }
    fn on_node(
        &mut self,
        node: &Self::Node,
    ) -> Option<Vec<Self::NextNode>>
    {
        self.labels.contains(node)
            .then_some(None)
            .unwrap_or_else(|| {
                let entry = self.vocab.get_vertex(node).unwrap();
                let next = self.entry_next(&entry);
                if self.entry_is_frequent(&entry)
                {
                    let key = entry.data.vertex_key();
                    self.labels.insert(key);
                }
                Some(next)
            })
    }
    fn begin_run(&mut self) {
        println!("Frequency Pass");
    }
    fn finish_run(&mut self) {
        let bottom = BottomUp::starting_nodes(&self.vocab);
        self.labels
            .extend(bottom.iter().map(HasVertexKey::vertex_key));
        self.status = ProcessStatus::Frequency;
    }
}
impl FrequencyCtx<'_>
{
    pub fn entry_next(
        &self,
        entry: &VertexCtx,
    ) -> Vec<NGramId>
    {
        TopDown::next_nodes(entry)
            .into_iter()
            .map(|(_, c)| c)
            .collect()
    }
    pub fn entry_is_frequent(
        &self,
        entry: &VertexCtx,
    ) -> bool
    {
        FrequencyCover::from_entry(self, entry)
            .iter()
            .any(|p|
                self.vocab.get_vertex(p).unwrap().count() < entry.count()
            )
            .then(|| println!("{}", entry.ngram))
            .is_some()
    }
}