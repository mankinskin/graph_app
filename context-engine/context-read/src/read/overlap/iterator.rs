use crate::read::{
    complement::ComplementBuilder,
    context::ReadContext,
    overlap::chain::OverlapChain,
};
use context_trace::{
    graph::vertex::wide::Wide,
    path::structs::rooted::role_path::PatternEndPath,
};
use derive_more::{
    Deref,
    DerefMut,
};

use super::{
    bundle::Bundle,
    chain::band::Band,
    generator::{
        ChainGenerator,
        ChainOp,
    },
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
                ChainOp::Expansion(next) => {
                    //println!("found overlap at {}: {:?}", start_bound, expansion);

                    // finish current bundle
                    if let Some(bundle) = self.bundle.take() {
                        let band = bundle.wrap_into_band(&mut self.trav);
                        self.chain.append(band);
                    }

                    // BACK CONTEXT FROM CACHE
                    // finish back context
                    let expansion = next.expansion;
                    let start_bound = next.start_bound;

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

                    let complement = ComplementBuilder::new(root, next)
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
        chain: OverlapChain,
    ) -> Self {
        Self {
            chain: ChainGenerator::new(trav, cursor, chain),
            bundle: None,
        }
    }
}
