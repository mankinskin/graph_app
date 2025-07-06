use context_trace::{
    self,
    path::structs::rooted::pattern_range::PatternRangePath,
};
use derive_new::new;

use crate::{
    context::ReadCtx,
    expansion::chain::{
        band::Band,
        expand::{
            ChainOp,
            ExpandCtx,
        },
        BandChain,
    },
};

#[derive(Debug, new)]
pub struct ChainCtx<'a> {
    pub trav: ReadCtx,
    pub cursor: &'a mut PatternRangePath,
    pub chain: BandChain,
}
impl<'a> ChainCtx<'a> {
    pub fn last(&self) -> &Band {
        &self.chain.last().unwrap().band
    }
}
impl<'a> Iterator for ChainCtx<'a> {
    type Item = Option<ChainOp>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(mut ctx) = ExpandCtx::try_new(self) {
            let op = ctx.next();
            if let Some(ChainOp::Expansion(_, expansion)) = &op {
                *self.cursor = expansion.path.clone();
            }
            Some(op)
        } else {
            None
        }
    }
}
