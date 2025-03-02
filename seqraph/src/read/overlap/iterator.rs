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
        pattern::Pattern,
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
        ExpansionLink,
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
                    let mut back_pattern = self.back_context_for_link(&next);

                    back_pattern.push(next.expansion);
                    let next_band = Band::from((0, back_pattern));
                    let end_bound = next.start_bound + next.expansion.width();
                    self.chain.append((end_bound, next_band));
                    Some(None)
                }
                ChainOp::Cap(cap) => {
                    // if not expandable, at band boundary -> add to bundle
                    // postfixes should always be first in the chain
                    let mut first_band = self.chain.pop_first().unwrap();
                    first_band.append(cap.expansion);
                    self.bundle = Some(if let Some(mut bundle) = self.bundle.take() {
                        bundle.add_pattern(first_band.pattern);
                        bundle
                    } else {
                        Bundle::new(first_band)
                    });
                    Some(None)
                }
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
    pub fn back_context_for_link(
        &mut self,
        next_link: &ExpansionLink,
    ) -> Pattern {
        //println!("building back context from path");
        //let (inner_back_ctx, _loc) = ctx
        //    .contexter::<SplitBack>()
        //    .try_context_path(
        //        //link.postfix_path.clone().into_context_path(),
        //        // FIXME: maybe mising root!!!
        //        link.postfix_path.clone().sub_path,
        //        //link.overlap,
        //    )
        //    .unwrap();

        let back_ctx = if let Some(last) = self.chain.bands.last() {
            let postfix_path = last.next_link.start_bound;
            //self.trav
            //    .read_pattern(last.back_context.borrow())
            //    .ok()
            //    //Some(self.graph.read_pattern(last.band.back_context.borrow()))
            //    .map(move |(back_ctx, _)| {
            //        last.back_context = vec![back_ctx];
            //        last.back_context.borrow()
            //    })
            Some(())
        } else {
            None
        }
        .unwrap_or_default();

        //DefaultDirection::context_then_inner(back_ctx, Child::new(0, 0)) //inner_back_ctx)
    }
}
