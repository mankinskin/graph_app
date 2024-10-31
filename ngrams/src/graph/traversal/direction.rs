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

pub trait TraversalDirection
{
    type Next;
    fn starting_nodes(vocab: &Vocabulary) -> VecDeque<NGramId>;
    fn next_nodes(entry: &VertexCtx<'_>) -> Vec<Self::Next>;
    fn order_top_bottom<T>(
        prev: T,
        next: T,
    ) -> (T, T);
}
impl TraversalDirection for BottomUp
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
            .filter(|(&id, p)| p.width == entry.data.width() + 1)
            .map(|(id, p)| {
                NGramId::new(
                    entry.vocab.containment.expect_key_for_index(id),
                    p.width(),
                )
            })
            .collect_vec()
    }
}
impl TraversalDirection for TopDown
{
    type Next = (usize, NGramId); // (off, id)
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
            .top_down_containment_nodes()
            .into_iter()
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
            .collect()

    }
}