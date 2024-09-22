use std::collections::VecDeque;

use itertools::Itertools;

use crate::graph::vocabulary::{
    entry::VertexCtx,
    NGramId,
    Vocabulary,
};
use seqraph::graph::vertex::{
    child::Child,
    key::VertexKey,
    wide::Wide,
    VertexIndex,
};

pub struct BottomUp;
pub struct TopDown;

pub trait TraversalPolicy
{
    type Next;
    fn starting_nodes(vocab: &Vocabulary) -> VecDeque<NGramId>;
    fn next_nodes(entry: &VertexCtx<'_>) -> Vec<Self::Next>;
    fn order_top_bottom<T>(
        prev: T,
        next: T,
    ) -> (T, T);
}
impl TraversalPolicy for BottomUp
{
    type Next = NGramId;
    fn starting_nodes(vocab: &Vocabulary) -> VecDeque<NGramId>
    {
        FromIterator::from_iter(vocab.leaves.iter().cloned())
    }
    fn order_top_bottom<T>(
        prev: T,
        next: T,
    ) -> (T, T)
    {
        (next, prev)
    }
    fn next_nodes(entry: &VertexCtx<'_>) -> Vec<Self::Next>
    {
        entry
            .data
            .parents
            .iter()
            .filter(|(&id, p)| p.width == entry.entry.ngram.len() + 1)
            .map(|(id, p)| {
                NGramId::new(
                    entry.vocab.containment.expect_key_for_index(id),
                    entry.data.width,
                )
            })
            .collect_vec()
    }
}
impl TraversalPolicy for TopDown
{
    type Next = (usize, NGramId);
    fn starting_nodes(vocab: &Vocabulary) -> VecDeque<NGramId>
    {
        FromIterator::from_iter(vocab.roots.iter().cloned())
    }
    fn order_top_bottom<T>(
        prev: T,
        next: T,
    ) -> (T, T)
    {
        (prev, next)
    }
    fn next_nodes(entry: &VertexCtx<'_>) -> Vec<Self::Next>
    {
        entry
            .data
            .children
            .iter()
            .flat_map(|(_, pat)| {
                pat.iter()
                    .enumerate()
                    .filter(|(subi, c)| c.width() + 1 == entry.entry.ngram.len())
                    .map(|(subi, c)| {
                        (
                            // sub index can be used as offset because child patterns have special structure
                            subi,
                            NGramId::new(
                                entry.vocab.containment.expect_key_for_index(c),
                                c.width(),
                            ),
                        )
                    })
            })
            .sorted_by_key(|&(off, _)| off)
            .collect_vec()
    }
}
