use crate::{
    context::ReadCtx,
    expansion::chain::{
        generator::ChainGenerator,
        BandCap,
        ChainOp,
    },
};
use context_insert::insert::{
    result::IndexWithPath,
    ToInsertCtx,
};
use context_trace::{
    graph::vertex::wide::Wide,
    path::{
        accessors::role::End,
        mutators::append::PathAppend,
        structs::{
            role_path::RolePath,
            sub_path::SubPath,
        },
    },
    trace::child::bands::{
        HasChildRoleIters,
        PostfixIterator,
    },
};

#[derive(Debug)]
pub struct ExpandCtx<'a, 'b> {
    pub gen: &'a ChainGenerator<'a, 'b>,
    pub postfix_path: RolePath<End>,
    pub postfix_iter: PostfixIterator<'a, ReadCtx>,
}
impl<'a, 'b> ExpandCtx<'a, 'b> {
    pub fn try_new(gen: &'a ChainGenerator<'a, 'b>) -> Option<Self> {
        let last_end = gen.last.postfix();
        let mut postfix_iter = last_end.postfix_iter(gen.ctx.trav.clone());
        if let Some((postfix_location, _)) = postfix_iter.next() {
            Some(Self {
                gen,
                postfix_path: RolePath::from(SubPath::new(
                    postfix_location.sub_index,
                )),
                postfix_iter,
            })
        } else {
            None
        }
    }
}
impl Iterator for ExpandCtx<'_, '_> {
    type Item = ChainOp;
    fn next(&mut self) -> Option<Self::Item> {
        self.postfix_iter.next().map(|(postfix_location, postfix)| {
            let last_end_bound = self.gen.last.end_bound;
            let start_bound = last_end_bound - postfix.width();
            self.postfix_path.path_append(postfix_location);
            match ToInsertCtx::<IndexWithPath>::insert(
                &self.gen.ctx.trav.graph,
                self.gen.ctx.cursor.clone(),
            ) {
                Ok(expansion) => ChainOp::Expansion(start_bound, expansion),
                Err(_) => ChainOp::Cap(BandCap {
                    postfix_path: self.postfix_path.clone(),
                    expansion: postfix,
                    start_bound,
                }),
            }
        })
    }
}
