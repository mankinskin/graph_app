use crate::{
    *,
    shared::*,
};
mod frequency;
mod wrappers;
mod traversal;

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
pub fn label_vocab(vocab: &Vocabulary) -> HashSet<VertexIndex> {
    //let roots = texts.iter().map(|s| *vocab.ids.get(s).unwrap()).collect_vec();
    let mut ctx = LabellingCtx::new(vocab);
    FrequencyCtx::from(&mut ctx).frequency_pass(vocab);
    //println!("{:#?}", ctx.labels);
    WrapperCtx::from(&mut ctx).wrapping_pass(vocab);
    //println!("{:#?}", ctx.labels);
    ctx.labels
}