pub mod bundle;
pub mod chain;

use crate::{
    complement::ComplementBuilder,
    context::ReadCtx,
    expansion::chain::{
        generator::ChainCtx,
        ChainOp,
        ExpansionLink,
    },
};
use chain::{
    band::Band,
    LinkedBands,
};

use bundle::Bundle;
use chain::generator::ChainGenerator;
use context_insert::insert::{
    result::IndexWithPath,
    ToInsertCtx,
};
use context_trace::{
    graph::{
        getters::ErrorReason,
        vertex::wide::Wide,
    },
    path::{
        accessors::{
            child::root::PatternRootChild,
            has_path::HasRolePath,
            role::{
                End,
                Start,
            },
        },
        mutators::append::PathAppend,
        structs::{
            query_range_path::FoldablePath,
            role_path::RolePath,
            rooted::{
                pattern_range::PatternRangePath,
                role_path::RootedRolePath,
            },
        },
    },
    trace::child::bands::HasChildRoleIters,
};
use derive_more::{
    Deref,
    DerefMut,
};
use itertools::{
    FoldWhile,
    Itertools,
};

#[derive(Debug, Deref, DerefMut)]
pub struct ExpansionIterator<'cursor> {
    #[deref]
    #[deref_mut]
    ctx: ChainCtx<'cursor>,
    bundle: Bundle,
    chain: LinkedBands,
}
impl<'a> Iterator for ExpansionIterator<'a> {
    type Item = ();

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(next_op) =
            ChainGenerator::new(&mut self.ctx, self.chain.last().unwrap().band)
                .next()
        {
            match next_op {
                ChainOp::Expansion(start_bound, next) => {
                    if self.bundle.len() > 1 {
                        self.bundle.wrap_into_band(&mut self.ctx.trav.graph);
                    }

                    // finish back context
                    let link = self.link_expansion(start_bound, &next);

                    // TODO:
                    // 1. walk overlaps at position
                    // 2. get furthest back facing overlap with position
                    // 3.

                    let &root = self
                        .chain
                        .bands
                        .first()
                        .expect("no overlaps in chain")
                        .pattern
                        .last()
                        .expect("empty pattern");

                    let complement = ComplementBuilder::new(root, link)
                        .build_context_index(&self.trav);

                    let back_pattern = vec![complement, next.index];
                    let next_band = Band::from((0, back_pattern));
                    let end_bound = start_bound + next.index.width();
                    self.chain.append((end_bound, next_band));
                    Some(())
                },
                ChainOp::Cap(cap) => {
                    match self.chain.ends_at(cap.start_bound) {
                        Some(_) => {
                            // if not expandable, at band boundary -> add to bundle
                            // postfixes should always be first in the chain
                            let mut first_band =
                                self.chain.pop_first().unwrap();
                            first_band.append(cap.expansion);
                            self.bundle.add_pattern(first_band.pattern);
                            Some(())
                        },
                        _ => None,
                    }
                },
            }
        } else {
            None
        }
    }
}
impl<'a> ExpansionIterator<'a> {
    pub fn new(
        trav: ReadCtx,
        cursor: &'a mut PatternRangePath,
    ) -> Self {
        let inner_cursor = cursor.clone();
        let first = match trav.insert_or_get_complete(inner_cursor) {
            Ok(Ok(IndexWithPath { index, path })) => {
                *cursor = path;
                index
            },
            Ok(Err(index)) => index,
            Err(ErrorReason::SingleIndex(c)) => c,
            Err(_) => cursor.start_index(&trav),
        };

        Self {
            chain: LinkedBands::new(first),
            ctx: ChainCtx::new(trav, cursor),
            bundle: Bundle::new(Band::from(first)),
        }
    }
    pub fn find_largest_bundle(mut self) -> Bundle {
        self.find(|_| false);
        self.bundle
    }
    fn link_expansion(
        &self,
        start_bound: usize,
        ext: &IndexWithPath,
    ) -> ExpansionLink {
        let advanced = &ext.path;
        let expansion = ext.index.clone();
        let adv_prefix =
            PatternRootChild::<Start>::pattern_root_child(advanced);
        // find prefix from advanced path in expansion index
        let mut prefix_iter = expansion.prefix_iter(self.trav.clone());
        let entry = prefix_iter.next().unwrap().0;
        let mut prefix_path = prefix_iter
            .fold_while(
                RootedRolePath::new(entry),
                |mut acc, (prefix_location, prefix)| {
                    acc.path_append(prefix_location);
                    if prefix == adv_prefix {
                        FoldWhile::Done(acc)
                    } else {
                        FoldWhile::Continue(acc)
                    }
                },
            )
            .into_inner();
        // append path <expansion to adv_prefix> to <adv_prefix to overlap>
        prefix_path.role_path.sub_path.path.extend(
            (advanced.role_path().clone() as RolePath<End>)
                .sub_path
                .path,
        );

        ExpansionLink {
            start_bound,
            prefix_path: prefix_path.role_path,
            expansion,
        }
    }
}
