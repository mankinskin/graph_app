pub mod cover;

use std::collections::VecDeque;

use derive_more::{
    Deref,
    DerefMut,
    From,
};
use itertools::Itertools;

use crate::graph::{
    containment::TextLocation,
    labelling::{
        LabellingCtx,
        frequency::cover::FrequencyCover,
    },
    traversal::{
        TraversalPass,
        BottomUp,
        TopDown,
        TraversalDirection,
    },
    vocabulary::{
        entry::{
            HasVertexEntries,
            VertexCtx,
        },
        NGramId,
        ProcessStatus,
        Vocabulary,
    },
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

#[derive(Debug, Deref, From, DerefMut)]
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
    fn on_node(
        &mut self,
        node: &Self::Node,
    ) -> Vec<Self::NextNode>
    {
        let entry = self.vocab.get_vertex(node).unwrap();
        let next = self.entry_next(&entry);
        if self.entry_is_frequent(&entry)
        {
            let key = entry.data.vertex_key();
            self.labels.insert(key);
        }
        next
    }
    fn run(&mut self)
    {
        println!("Frequency Pass");
        let start = TopDown::starting_nodes(&self.vocab);
        let mut queue = Queue::new(
            VecDeque::default(),
        );
        self.labels.extend(start.iter().map(HasVertexKey::vertex_key));
        for node in start.into_iter()
        {
            queue.extend_queue(
                self.on_node(&node)
            );
        }
        while let Some(node) = queue.pop_front()
        {
            if !self.labels.contains(&node)
            {
                queue.extend_queue(
                    self.on_node(&node)
                );
            }
        }
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
#[derive(Debug, Deref, DerefMut)]
pub struct Queue {
    pub queue: VecDeque<NGramId>,
}

impl Queue
{
    pub fn new<T: IntoIterator<Item = NGramId>>(
        iter: T,
    ) -> Self
    {
        let mut v = Self {
            queue: VecDeque::default(),
        };
        v.extend_queue(iter);
        v
    }
    pub fn extend_queue<T: IntoIterator<Item = NGramId>>(
        &mut self,
        iter: T,
    )
    {
        self.queue.extend(iter);
        self.queue = self
            .queue
            .drain(..)
            .sorted_by_key(|i| std::cmp::Reverse(i.width()))
            .dedup()
            .collect();
    }
}
