use crate::read::{
    context::ReadContext,
    overlap::chain::OverlapChain,
};
use derive_more::{
    Deref,
    DerefMut,
};
use hypercontext_api::{
    self,
    graph::vertex::{
        child::Child,
        wide::Wide,
    },
    path::structs::rooted::role_path::PatternEndPath,
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
    type Item = Child;

    fn next(&mut self) -> Option<Self::Item> {
        // find expandable postfix, may append postfixes in overlap chain
        //println!("read next overlap with {:#?}", cache.last);
        if let Some(next_op) = self.chain.next() {
            match next_op {
                ChainOp::Expand(next) => {
                    //println!("found overlap at {}: {:?}", start_bound, expansion);

                    // finish current bundle
                    if let Some(bundle) = self.bundle {
                        self.chain.append((self.trav, bundle));
                    }

                    // BACK CONTEXT FROM CACHE
                    // finish back context
                    let back_pattern =
                        self.chain
                            .back_context_for_link(&mut self.trav, next.start_bound, &next);
                    back_pattern.append(next.expansion);

                    let next_band = Band::from(back_pattern);
                    let end_bound = next.start_bound + next.expansion.width();
                    self.chain.append((end_bound, next_band));

                    Some(())
                }
                ChainOp::Cap(cap) => {
                    // if not expandable, at band boundary -> add to bundle
                    // postfixes should always be first in the chain
                    let mut first_band = self.chain.pop_first().unwrap();
                    first_band.append(cap.expansion);
                    self.bundle = Some(self.bundle.map_or_else(
                        || Bundle::new(first_band),
                        |bundle| {
                            bundle.add_pattern(first_band.pattern);
                            bundle
                        },
                    ));
                    Some(())
                }
            }
        } else {
            self.bundle
                .take()
                .map(|b| self.chain.append((self.trav, b)));
            self.chain.close(self.trav);
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
