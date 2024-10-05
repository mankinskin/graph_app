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
        let next = TopDown::next_nodes(entry);
        next.into_iter().map(|(_, c)| c).collect()
    }
    fn next_nodes(entry: &VertexCtx) -> Vec<(usize, NGramId)>
    {
        entry
            .direct_parents()
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
    pub fn entry_is_frequent(
        &self,
        entry: &VertexCtx,
    ) -> bool
    {
        let mut cover: HashSet<_> = Default::default();
        let mut occ_set: HashSet<_> = Default::default(); //entry.occurrences.clone();
        let mut queue: VecDeque<_> =
            FromIterator::from_iter(Self::next_nodes(entry));
        while let Some((off, p)) = queue.pop_front()
        {
            let pe = entry.vocab.get_vertex(&p).unwrap();
            if let Some(occ) = {
                if self.labels.contains(&p.vertex_key())
                {
                    let occ: HashSet<_> = pe
                        .occurrences
                        .iter()
                        .map(|loc| TextLocation::new(loc.texti, loc.x + off))
                        .collect();
                    (occ.difference(&occ_set).count() != 0).then_some(occ)
                }
                else
                {
                    None
                }
            }
            {
                cover.insert((p, pe.ngram.clone()));
                occ_set.extend(&occ);
            }
            else
            {
                queue.extend(
                    Self::next_nodes(&pe)
                        .into_iter()
                        .map(|(o, p)| (o + off, p)),
                );
            }
        }
        let f = cover.iter().any(|(p, _)| {
            entry.vocab.get_vertex(p).unwrap().count() < entry.count()
        });
        if f
        {
            println!("{}", entry.ngram);
        }
        f
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
            let index = entry.data.vertex_key();
            self.labels.insert(index);
            next
        }
        else
        {
            next
        }
    }
    pub fn frequency_pass(&mut self)
    {
        println!("Frequency Pass");
        let start = TopDown::starting_nodes(&self.vocab);
        self.labels
            .extend(start.iter().map(HasVertexKey::vertex_key));
        let mut queue = Queue::new(VecDeque::default(), &self.vocab);
        for node in start
        {
            let next = self.on_node(&node);
            queue.extend_queue(next, &self.vocab);
        }
        while let Some(node) = queue.pop_front()
        {
            if !self.labels.contains(&node)
            {
                let next = self.on_node(&node);
                //layer.extend(next);
                queue.extend_queue(next, &self.vocab);
            }
        }
        let bottom = BottomUp::starting_nodes(&self.vocab);
        self.labels
            .extend(bottom.iter().map(HasVertexKey::vertex_key));
        self.status = ProcessStatus::Frequency;
    }
}
#[derive(Debug, Deref, DerefMut)]
pub struct Queue(pub VecDeque<NGramId>);

impl Queue
{
    pub fn new<T: IntoIterator<Item = NGramId>>(
        iter: T,
        vocab: &Vocabulary,
    ) -> Self
    {
        let mut v = Self(VecDeque::default());
        v.extend_queue(iter, vocab);
        v
    }
    pub fn extend_queue<T: IntoIterator<Item = NGramId>>(
        &mut self,
        iter: T,
        vocab: &Vocabulary,
    )
    {
        self.0.extend(iter);
        self.0 = self
            .0
            .drain(..)
            .sorted_by_key(|i| std::cmp::Reverse(i.width()))
            .dedup()
            .collect();
    }
}
