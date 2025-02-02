use std::{
    borrow::Borrow,
    collections::BTreeMap,
};

use derive_more::derive::{
    Deref,
    DerefMut,
};
use itertools::Itertools;
use tracing::instrument;

use crate::{
    insert::{
        direction::InsertDirection,
        HasInsertContext,
    },
    read::{
        bundle::band::OverlapBundle,
        overlap::Overlap,
        reader::context::ReadContext,
    },
};
use hypercontext_api::{
    graph::{
        kind::DefaultDirection,
        vertex::{
            child::Child,
            pattern::Pattern,
        },
    },
    split::cache::ctx,
    tests::graph::context_mut,
    traversal::traversable::TraversableMut,
};

use super::OverlapLink;

#[derive(Default, Clone, Debug, Deref, DerefMut)]
pub struct OverlapChain {
    pub chain: BTreeMap<usize, Overlap>,
}

impl OverlapChain {
    pub fn back_context_for_link(
        &mut self,
        mut trav: impl TraversableMut,
        start_bound: usize,
        next_link: &OverlapLink,
    ) -> Pattern {
        let past_ctx = self.take_past_context_pattern(&mut trav, start_bound);

        if let Some((past_end_bound, past_ctx)) = past_ctx {
            //println!("reusing back context {past_end_bound}: {:#?}", past_ctx);
            if past_end_bound == start_bound {
                past_ctx
            } else {
                assert!(past_end_bound < start_bound);
                panic!("Shouldn't this be impossible?!");
            }
        } else {
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

            let back_ctx = if let Some((_, last)) = self.chain.iter_mut().last() {
                trav.graph_mut()
                    .index_pattern(last.band.back_context.borrow())
                    .ok()
                    //Some(self.graph.read_pattern(last.band.back_context.borrow()))
                    .map(move |(back_ctx, _)| {
                        last.band.back_context = vec![back_ctx];
                        last.band.back_context.borrow()
                    })
            } else {
                None
            }
            .unwrap_or_default();
            DefaultDirection::context_then_inner(back_ctx, Child::new(0, 0)) //inner_back_ctx)
        }
    }
    //#[instrument(skip(self, start_bound, ctx))]
    pub fn take_past_context_pattern(
        &mut self,
        trav: impl TraversableMut,
        start_bound: usize,
    ) -> Option<(usize, Pattern)> {
        let mut past = self.take_past(start_bound);
        match past.chain.len() {
            0 => None,
            1 => {
                let (end_bound, past) = past.chain.pop_last().unwrap();
                Some((end_bound, past.band.into_pattern()))
            }
            _ => {
                let past_key = *past.chain.keys().last().unwrap();
                let past_index = past.close(trav).unwrap();
                Some((past_key, vec![past_index]))
            }
        }
    }
    pub fn add_bundle(
        &mut self,
        trav: impl TraversableMut,
        end_bound: usize,
        bundle: OverlapBundle,
    ) {
        self.insert(
            end_bound,
            Overlap {
                link: None,
                band: bundle.write_band(trav),
            },
        );
    }
    pub fn add_overlap(
        &mut self,
        end_bound: usize,
        overlap: Overlap,
    ) -> Result<(), ()> {
        // postfixes should always start at first end bounds in the chain
        if self.chain.get(&end_bound).is_some() {
            Err(())
        } else {
            self.chain.insert(end_bound, overlap);
            Ok(())
        }
    }
    #[instrument(skip(self, trav))]
    pub fn close(
        self,
        trav: impl TraversableMut,
    ) -> Option<Child> {
        //println!("closing {:#?}", self);
        let mut chain_iter = self.chain.into_iter();
        let first_band: Overlap = chain_iter.next()?.1;
        // this part should create the missing front contexts of each band.
        // this should however start at the end of the chain and work backwards
        // we need to get paths to the overlaps with each previous band
        // then we can use each of these paths to create a partition for the front context of the overlap within the last index
        // this can be appended to the back of the band
        let (mut bundle, prev_band, _) = {
            chain_iter.fold(
                (vec![], first_band, None),
                |(mut bundle, prev_band, prev_ctx), (_end_bound, overlap)| {
                    // index context of prefix
                    let ctx_child = if let Some(link) = overlap.link.as_ref() {
                        todo!("implement front context indexing");
                        //reader
                        //    .contexter::<SplitFront>()
                        //    .try_context_path(
                        //        link.prefix_path
                        //            .get_path()
                        //            .unwrap()
                        //            .clone()
                        //            .into_context_path(),
                        //        //node.overlap,
                        //    )
                        //    .map(|(ctx, _)| ctx)
                        None
                    } else {
                        None
                    };

                    bundle.push(prev_band);
                    (
                        bundle,
                        overlap,
                        // join previous and current context into
                        prev_ctx
                            .map(|prev| {
                                ctx_child
                                    .map(|ctx_child| {
                                        trav.read_pattern(vec![prev, ctx_child]).unwrap()
                                    })
                                    .or(prev)
                            })
                            .or(ctx_child),
                    )
                },
            )
        };
        bundle.push(prev_band);
        let bundle = bundle
            .into_iter()
            .map(|overlap| overlap.band.into_pattern())
            .collect_vec();
        let index = trav.graph_mut().insert_patterns(bundle);
        //println!("close result: {:?}", index);
        Some(index)
    }
    #[instrument(skip(self, end_bound))]
    pub fn take_past(
        &mut self,
        end_bound: usize,
    ) -> OverlapChain {
        let mut past = self.chain.split_off(&end_bound);
        std::mem::swap(&mut self.chain, &mut past);
        Self { chain: past }
    }
}
