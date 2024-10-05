use std::collections::VecDeque;

use derive_more::{
    Deref,
    DerefMut,
    From,
};
use itertools::Itertools;

use crate::graph::{
    containment::TextLocation,
    labelling::LabellingCtx,
    traversal::{
        BottomUp,
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

#[derive(Debug, Default, Clone, Deref)]
pub struct FrequencyCover {
    #[deref]
    cover: HashSet<NGramId>,
}
impl FrequencyCover {
    pub fn from_entry(
        ctx: &LabellingCtx,
        entry: &VertexCtx,
    ) -> Self {
        let mut cover: HashSet<_> = Default::default();
        let mut occ_set: HashSet<_> = Default::default(); //entry.occurrences.clone();
        let mut queue: VecDeque<_> =
            FromIterator::from_iter(Self::next_parent_offsets(entry));
        while let Some((off, p)) = queue.pop_front()
        {
            let pe = entry.vocab.get_vertex(&p).unwrap();
            let diff = Self::new_occurrences(ctx, off, &pe, &occ_set);
            if diff.is_empty() {
                queue.extend(
                    Self::next_parent_offsets(&pe)
                        .into_iter()
                        .map(|(o, p)| (o + off, p)),
                );
            }
            else
            {
                cover.insert(p);
                occ_set.extend(&diff);
            }
        }
        Self { cover }
    }
    fn next_parent_offsets(entry: &VertexCtx) -> Vec<(usize, NGramId)>
    {
        entry.data.parents
            .iter()
            .flat_map(|(&id, p)| {
                p.pattern_indices.iter().map(move |ploc| {
                    (
                        entry.vocab.containment.expect_child_offset(
                            &ChildLocation::new(
                                Child::new(id, p.width),
                                ploc.pattern_id,
                                ploc.sub_index,
                            ),
                        ),
                        NGramId::new(
                            entry.vocab.containment.expect_key_for_index(id),
                            p.width,
                        ),
                    )
                })
            })
            .collect_vec()
    }
    pub fn new_occurrences(
        ctx: &LabellingCtx,
        offset: usize,
        parent_entry: &VertexCtx,
        occ_set: &HashSet<TextLocation>,
    ) -> HashSet<TextLocation>
    {
        ctx.labels.contains(&parent_entry.vertex_key())
            .then(|| {
                let occ: HashSet<_> = parent_entry
                    .occurrences
                    .iter()
                    .map(|loc| TextLocation::new(loc.texti, loc.x + offset))
                    .collect();
                occ.difference(occ_set).copied().collect()
            })
            .unwrap_or_default()
    }
}
#[derive(Debug, Deref, From, DerefMut)]
pub struct FrequencyCtx<'b>
{
    #[deref]
    #[deref_mut]
    pub ctx: &'b mut LabellingCtx,
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
    pub fn on_node(
        &mut self,
        node: &VertexKey,
    ) -> Vec<NGramId>
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
    pub fn frequency_pass(&mut self)
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
