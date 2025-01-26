use std::collections::BTreeMap;

use derive_more::derive::{Deref, DerefMut};
use itertools::Itertools;
use tracing::instrument;

use crate::read::{
    bands::overlaps::overlap::Overlap,
    reader::context::ReadContext,
};
use hypercontext_api::{
    graph::vertex::child::Child,
    split::side::SplitFront,
    traversal::traversable::TraversableMut,
};

#[derive(Default, Clone, Debug, Deref, DerefMut)]
pub struct OverlapChain {
    pub chain: BTreeMap<usize, Overlap>,
}

impl OverlapChain {
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
    #[instrument(skip(self, reader))]
    pub fn close(
        self,
        reader: &mut ReadContext,
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
                        if let Some(prev) = prev_ctx {
                            Some(if let Some(ctx_child) = ctx_child {
                                reader.read_pattern(vec![prev, ctx_child]).unwrap()
                            } else {
                                prev
                            })
                        } else {
                            ctx_child
                        },
                    )
                },
            )
        };
        bundle.push(prev_band);
        let bundle = bundle
            .into_iter()
            .map(|overlap| overlap.band.into_pattern(reader))
            .collect_vec();
        let index = reader.graph_mut().insert_patterns(bundle);
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
