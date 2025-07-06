use context_trace::{
    self,
    path::structs::rooted::pattern_range::PatternRangePath,
};
use derive_new::new;

use crate::{
    context::ReadCtx,
    expansion::chain::{
        band::Band,
        expand::ExpandCtx,
        BandChain,
        ChainOp,
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
    pub fn find_next_operation(&mut self) -> Option<ChainOp> {
        if let Some(mut ctx) = ExpandCtx::try_new(self) {
            while let Some(op) = ctx.next() {
                match &op {
                    ChainOp::Expansion(_, expansion) => {
                        *self.cursor = expansion.path.clone();
                        return Some(op);
                    },
                    ChainOp::Cap(cap) =>
                        if let Some(_) = self.chain.ends_at(cap.start_bound) {
                            return Some(op);
                        },
                }
            }
        }
        None
    }
}
impl<'a> Iterator for ChainCtx<'a> {
    type Item = ChainOp;

    fn next(&mut self) -> Option<Self::Item> {
        self.find_next_operation()
    }
}
