use crate::{
    *,
    shared::*,
};
mod frequency;
mod wrappers;

use {
    frequency::*,
    wrappers::*,
};

#[derive(Debug, Clone)]
pub struct LabellingCtx<'a> {
    pub vocab: &'a Vocabulary,
    pub labels: HashSet<VertexIndex>,
}
impl<'a> LabellingCtx<'a> {
    pub fn new(vocab: &'a Vocabulary) -> Self {
        let mut ctx = Self {
            vocab,
            labels: Default::default(),
            //FromIterator::from_iter(
            //    vocab.leaves.iter().chain(
            //        vocab.roots.iter()
            //    )
            //    .cloned(),
            //),
        };
        ctx
    }
}
pub struct BottomUp;
pub struct TopDown;
pub trait TraversalPolicy {
    type Next;
    fn starting_nodes(vocab: &Vocabulary) -> VecDeque<VertexIndex>;
    fn next_nodes(entry: &VertexCtx<'_>) -> Vec<Self::Next>;
    fn order_top_bottom<T>(prev: T, next: T) -> (T, T);
}
impl TraversalPolicy for BottomUp {
    type Next = VertexIndex;
    fn starting_nodes(vocab: &Vocabulary) -> VecDeque<VertexIndex> {
        FromIterator::from_iter(
            vocab.leaves.iter().cloned()
        )
    }
    fn order_top_bottom<T>(prev: T, next: T) -> (T, T) {
        (next, prev)
    }
    fn next_nodes(entry: &VertexCtx<'_>) -> Vec<Self::Next> {
        entry.data.parents.iter()
            .filter_map(|(&id, p)|
                (p.width == entry.entry.ngram.len() + 1).then(|| id)
                //Some(id)
            ).collect_vec()
    }
}
impl TraversalPolicy for TopDown {
    type Next = (usize, Child);
    fn starting_nodes(vocab: &Vocabulary) -> VecDeque<VertexIndex> {
        FromIterator::from_iter(
            vocab.roots.iter().cloned()
        )
    }
    fn order_top_bottom<T>(prev: T, next: T) -> (T, T) {
        (prev, next)
    }
    fn next_nodes(entry: &VertexCtx<'_>) -> Vec<Self::Next> {
        entry.data.children.iter().flat_map(|(_, pat)|
            pat.iter().enumerate().filter_map(|(off, c)| 
                (c.width() + 1 == entry.entry.ngram.len()).then(||
                    (off, *c)
                )
            )
        ).collect_vec()
    }
}
pub fn label_vocab(vocab: &Vocabulary) -> HashSet<VertexIndex> {
    //let roots = texts.iter().map(|s| *vocab.ids.get(s).unwrap()).collect_vec();
    let mut ctx = LabellingCtx::new(vocab);
    FrequencyCtx::from(&mut ctx).frequency_pass(vocab);
    println!("{:#?}", ctx.labels);
    WrapperCtx::from(&mut ctx).wrapping_pass(vocab);
    //println!("{:#?}", ctx.labels);
    ctx.labels
}