use crate::{
    complement::ComplementBuilder,
    context::ReadCtx,
    overlap::{
        bands::{
            band::Band,
            generator::{
                ChainGenerator,
                ChainOp,
                ExpansionLink,
            },
            LinkedBands,
        },
        bundle::Bundle,
    },
};
use context_insert::insert::{
    result::IndexWithPath,
    ToInsertCtx,
};
use context_trace::{
    graph::vertex::wide::Wide,
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
pub struct ExpansionIterator<'a> {
    #[deref]
    #[deref_mut]
    chain: ChainGenerator<'a>,
    bundle: Bundle,
}
impl<'a> Iterator for ExpansionIterator<'a> {
    type Item = ();

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(next_op) = self.chain.next() {
            match next_op {
                ChainOp::Expansion(start_bound, next) => {
                    if self.bundle.len() > 1 {
                        self.bundle.wrap_into_band(&mut self.chain.trav.graph);
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
                    // if not expandable, at band boundary -> add to bundle
                    // postfixes should always be first in the chain
                    let mut first_band = self.chain.pop_first().unwrap();
                    first_band.append(cap.expansion);
                    self.bundle.add_pattern(first_band.pattern);
                    Some(())
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
            Ok(IndexWithPath { index, path }) => {
                *cursor = path;
                index
            },
            Err(_) => cursor.start_index(&trav),
        };

        let chain = LinkedBands::new(first);
        Self {
            chain: ChainGenerator::new(trav, cursor, chain),
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
