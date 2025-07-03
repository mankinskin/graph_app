use crate::{
    context::ReadCtx,
    expansion::chain::{
        band::Band,
        expand::ExpandCtx,
        ChainOp,
    },
};

use context_trace::path::structs::rooted::pattern_range::PatternRangePath;
use derive_new::new;

#[derive(Debug, new)]
pub struct ChainCtx<'cursor> {
    pub trav: ReadCtx,
    pub cursor: &'cursor mut PatternRangePath,
}
#[derive(Debug, new)]
pub struct ChainGenerator<'a, 'cursor> {
    pub ctx: &'a mut ChainCtx<'cursor>,
    pub last: &'a Band,
}
impl<'a, 'b> Iterator for ChainGenerator<'a, 'b> {
    type Item = ChainOp;

    fn next(&mut self) -> Option<Self::Item> {
        ExpandCtx::try_new(self)
            .and_then(|mut ctx| ctx.next())
            .map(|op| {
                if let ChainOp::Expansion(_, expansion) = &op {
                    *self.ctx.cursor = expansion.path.clone();
                }
                op
            })
    }
}
