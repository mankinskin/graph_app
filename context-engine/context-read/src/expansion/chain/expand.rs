use crate::{
    context::ReadCtx,
    expansion::{
        chain::{
            link::{
                BandCap,
                BandExpansion,
            },
            ChainOp,
        },
        ExpansionCtx,
    },
};
use context_insert::insert::ToInsertCtx;
use context_trace::{
    graph::{
        getters::IndexWithPath,
        vertex::wide::Wide,
    },
    path::{
        mutators::append::PathAppend,
        structs::rooted::role_path::IndexEndPath,
    },
    trace::child::bands::{
        HasChildRoleIters,
        PostfixIterator,
    },
};
use tracing::debug;

#[derive(Debug)]
pub struct ExpandCtx<'a> {
    pub ctx: &'a ExpansionCtx<'a>,
    pub postfix_path: IndexEndPath,
    pub postfix_iter: PostfixIterator<'a, ReadCtx>,
}
impl<'a> ExpandCtx<'a> {
    pub fn try_new(ctx: &'a ExpansionCtx<'a>) -> Option<Self> {
        debug!("Try new ExpandCtx");
        let last_end = ctx.last().postfix();
        let mut postfix_iter = last_end.postfix_iter(ctx.ctx.clone());
        if let Some((postfix_location, _)) = postfix_iter.next() {
            Some(Self {
                ctx,
                postfix_path: IndexEndPath::from(postfix_location),
                postfix_iter,
            })
        } else {
            None
        }
    }
}
impl Iterator for ExpandCtx<'_> {
    type Item = ChainOp;
    fn next(&mut self) -> Option<Self::Item> {
        debug!("ExpandCtx::next");
        self.postfix_iter.next().map(|(postfix_location, postfix)| {
            let last_end_bound = self.ctx.last().end_bound;
            let start_bound = last_end_bound - postfix.width();
            self.postfix_path.path_append(postfix_location);
            match ToInsertCtx::<IndexWithPath>::insert(
                &self.ctx.ctx.graph,
                self.ctx.cursor.cursor.clone(),
            ) {
                Ok(expansion) => ChainOp::Expansion(BandExpansion {
                    start_bound,
                    expansion,
                    postfix_path: self.postfix_path.clone(),
                }),
                Err(_) => ChainOp::Cap(BandCap {
                    postfix_path: self.postfix_path.clone(),
                    expansion: postfix,
                    start_bound,
                }),
            }
        })
    }
}
