use crate::{
    complement::ComplementBuilder,
    context::ReadContext,
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
use context_insert::insert::result::IndexWithPath;
use context_trace::{
    graph::vertex::{
        child::Child,
        wide::Wide,
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
            role_path::RolePath,
            rooted::{
                pattern_range::PatternRangePath,
                role_path::{
                    PatternEndPath,
                    RootedRolePath,
                },
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
    bundle: Option<Bundle>,
}
impl<'a> Iterator for ExpansionIterator<'a> {
    type Item = Option<Bundle>;

    fn next(&mut self) -> Option<Self::Item> {
        // find expandable postfix, may append postfixes in overlap chain
        //println!("read next overlap with {:#?}", cache.last);
        if let Some(next_op) = self.chain.next() {
            match next_op {
                ChainOp::Expansion(start_bound, next) => {
                    //println!("found overlap at {}: {:?}", start_bound, expansion);

                    // finish current bundle
                    if let Some(bundle) = self.bundle.take() {
                        let band = bundle.wrap_into_band(&mut self.trav.graph);
                        self.chain.append(band);
                    }

                    // BACK CONTEXT FROM CACHE
                    // finish back context
                    let link = self.link_expansion(start_bound, next);

                    // TODO:
                    // 1. walk overlaps at position
                    // 2. get furthest back facing overlap with position
                    // 3.

                    // earliest overlap
                    //
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
                    //let mut back_pattern = self.back_context_for_link(&next);

                    let back_pattern = vec![complement, expansion];
                    let next_band = Band::from((0, back_pattern));
                    let end_bound = start_bound + expansion.width();
                    self.chain.append((end_bound, next_band));
                    Some(None)
                },
                ChainOp::Cap(cap) => {
                    // if not expandable, at band boundary -> add to bundle
                    // postfixes should always be first in the chain
                    let mut first_band = self.chain.pop_first().unwrap();
                    first_band.append(cap.expansion);
                    self.bundle =
                        Some(if let Some(mut bundle) = self.bundle.take() {
                            bundle.add_pattern(first_band.pattern);
                            bundle
                        } else {
                            Bundle::new(first_band)
                        });
                    Some(None)
                },
            }
        } else {
            None
        }
    }
}
impl<'a> ExpansionIterator<'a> {
    pub fn new(
        trav: ReadContext,
        cursor: &'a mut PatternEndPath,
        chain: LinkedBands,
    ) -> Self {
        Self {
            chain: ChainGenerator::new(trav, cursor, chain),
            bundle: None,
        }
    }
    fn link_expansion(
        &self,
        start_bound: usize,
        ext: IndexWithPath,
    ) -> ExpansionLink {
        let IndexWithPath {
            index: expansion,
            path: advanced,
        } = ext;
        let adv_prefix =
            PatternRootChild::<Start>::pattern_root_child(&advanced);
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
