use std::collections::VecDeque;

use derive_more::{
    Deref,
    DerefMut,
    From,
};
use itertools::Itertools;

use seqraph::{
    vertex::{
        child::Child,
        location::child::ChildLocation,
        VertexIndex,
    },
    HashSet,
};

use crate::graph::traversal::{
    BottomUp,
    TopDown,
    TraversalPolicy,
};
use crate::graph::{
    containment::TextLocation,
    labelling::LabellingCtx,
    vocabulary::{
        IndexVocab,
        VertexCtx,
        Vocabulary,
    },
};

#[derive(Debug, Deref, From, DerefMut)]
pub struct FrequencyCtx<'a>
{
    #[deref]
    #[deref_mut]
    ctx: &'a mut LabellingCtx,
}
impl<'a> FrequencyCtx<'a>
{
    pub fn entry_next(
        &mut self,
        entry: &VertexCtx,
    ) -> Vec<VertexIndex>
    {
        let next = TopDown::next_nodes(entry);
        next.into_iter().map(|(_, c)| c.index).collect()
    }
    fn next_nodes(entry: &VertexCtx) -> Vec<(usize, VertexIndex)>
    {
        entry
            .direct_parents()
            .iter()
            .map(|(&id, p)| {
                p.pattern_indices.iter().map(move |ploc| {
                    (
                        entry.vocab.graph.expect_child_offset(
                            &ChildLocation::new(
                                Child::new(id, p.width),
                                ploc.pattern_id,
                                ploc.sub_index,
                            ),
                        ),
                        id,
                    )
                })
            })
            .flatten()
            .collect_vec()
    }
    pub fn entry_is_frequent(
        &mut self,
        entry: &VertexCtx,
    ) -> bool
    {
        let mut cover: HashSet<_> = Default::default();
        let mut occ_set: HashSet<_> = Default::default(); //entry.occurrences.clone();
        let mut queue: VecDeque<_> =
            FromIterator::from_iter(Self::next_nodes(&entry));
        while let Some((off, p)) = queue.pop_front()
        {
            let pe = entry.vocab.get(&p).unwrap();
            if let Some(occ) = {
                if self.labels.contains(&p)
                {
                    let occ: HashSet<_> = pe
                        .occurrences
                        .iter()
                        .map(|loc| TextLocation::new(loc.texti, loc.x + off))
                        .collect();
                    (occ.difference(&occ_set).count() != 0).then(|| occ)
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
        let f = cover
            .iter()
            .any(|(p, _)| entry.vocab.get(p).unwrap().count() < entry.count());
        if f
        {
            println!("{}", entry.ngram);
        }
        f
    }
    pub fn on_node(
        &mut self,
        entry: &VertexCtx,
    ) -> Vec<VertexIndex>
    {
        if self.entry_is_frequent(entry)
        {
            self.labels.insert(entry.data.index);
        }

        self.entry_next(entry)
    }
    pub fn frequency_pass(
        &mut self,
        vocab: &Vocabulary,
    )
    {
        let start = TopDown::starting_nodes(vocab);
        self.labels.extend(start.iter());
        let mut queue = Queue::new(VecDeque::default(), vocab);
        for node in start
        {
            let next = self.on_node(&vocab.get(&node).unwrap());
            queue.extend_queue(next, &vocab);
        }
        while let Some(node) = queue.pop_front()
        {
            if !self.labels.contains(&node)
            {
                let next = self.on_node(&vocab.get(&node).unwrap());
                //layer.extend(next);
                queue.extend_queue(next, &vocab);
            }
        }
        self.labels.extend(BottomUp::starting_nodes(vocab));
    }
}
#[derive(Debug, Deref, DerefMut)]
pub struct Queue(pub VecDeque<usize>);

impl Queue
{
    pub fn new<T: IntoIterator<Item = usize>>(
        iter: T,
        vocab: &Vocabulary,
    ) -> Self
    {
        let mut v = Self(VecDeque::default());
        v.extend_queue(iter, vocab);
        v
    }
    pub fn extend_queue<T: IntoIterator<Item = usize>>(
        &mut self,
        iter: T,
        vocab: &Vocabulary,
    )
    {
        self.0.extend(iter);
        self.0 = self
            .0
            .drain(..)
            .sorted_by_key(|i| {
                (std::cmp::Reverse(vocab.graph.expect_index_width(i)), *i)
            })
            .dedup()
            .collect();
    }
}
