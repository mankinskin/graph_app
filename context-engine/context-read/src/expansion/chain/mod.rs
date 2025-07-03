pub mod band;
pub mod expand;
pub mod generator;
use std::collections::{
    BTreeSet,
    VecDeque,
};

use band::Band;
use context_insert::insert::result::IndexWithPath;
use context_trace::{
    self,
    graph::vertex::{
        child::Child,
        wide::Wide,
    },
    path::{
        accessors::role::{
            End,
            Start,
        },
        structs::role_path::RolePath,
    },
};
use derive_more::From;

use crate::expansion::chain::band::BandCtx;

pub trait ChainAppendage: Sized {
    fn append_to_chain(
        self,
        chain: &mut LinkedBands,
    ) {
        chain.bands.insert(self.into_band());
    }
    fn into_band(self) -> Band;
}
impl ChainAppendage for Band {
    fn into_band(self) -> Band {
        self
    }
}
impl ChainAppendage for (usize, Band) {
    fn into_band(self) -> Band {
        self.1
    }
}
#[derive(Debug, From)]
pub enum ChainOp {
    Expansion(usize, IndexWithPath),
    Cap(BandCap),
}
#[derive(Debug)]
pub struct BandCap {
    pub postfix_path: RolePath<End>,
    pub expansion: Child,
    pub start_bound: usize,
}
#[derive(Debug)]
pub struct ExpansionLink {
    pub prefix_path: RolePath<Start>,
    pub expansion: Child,
    pub start_bound: usize,
}
#[derive(Clone, Debug)]
pub struct OverlapLink {
    pub postfix_path: RolePath<End>, // location of postfix/overlap in first index
    pub prefix_path: RolePath<Start>, // location of prefix/overlap in second index
}

#[derive(Default, Clone, Debug)]
pub struct LinkedBands {
    pub bands: BTreeSet<Band>,
    pub links: VecDeque<OverlapLink>,
}
impl LinkedBands {
    pub fn new(index: Child) -> Self {
        Self {
            bands: Some(Band {
                pattern: vec![index],
                start_bound: 0,
                end_bound: index.width(),
            })
            .into_iter()
            .collect(),
            links: Default::default(),
        }
    }
    pub fn ends_at(
        &self,
        bound: usize,
    ) -> Option<BandCtx<'_>> {
        let band = self.bands.get(&bound)?;

        Some(BandCtx {
            band,
            back_link: self.links.iter().last(),
            front_link: None,
        })
    }
    pub fn last(&self) -> Option<BandCtx<'_>> {
        self.bands.iter().last().map(|band| BandCtx {
            band,
            back_link: self.links.iter().last(),
            front_link: None,
        })
    }
    pub fn append(
        &mut self,
        band: impl ChainAppendage,
    ) {
        band.append_to_chain(self);
    }
    pub fn pop_first(&mut self) -> Option<Band> {
        self.links.pop_front();
        self.bands.pop_first()
    }

    //pub fn append_overlap(
    //    &mut self,
    //    overlap: OverlapBand,
    //) -> Result<(), ()> {
    //}
    //pub fn take_past(
    //    &mut self,
    //    bound: usize,
    //) -> LinkedBands {
    //    let mut past = self.chain.split_off(&bound);
    //    std::mem::swap(&mut self.chain, &mut past);
    //    Self {
    //        chain: past,
    //        end_bound: bound,
    //    }
    //}

    //#[instrument(skip(self, trav))]
    //pub fn close(
    //    self,
    //    trav: impl HasGraphMut,
    //) -> Option<Child> {
    //    //println!("closing {:#?}", self);
    //    let mut chain_iter = self.chain.into_iter();
    //    let first_band: Overlap = chain_iter.next()?.1;
    //    // this part should create the missing front contexts of each band.
    //    // this should however start at the end of the chain and work backwards
    //    // we need to get paths to the overlaps with each previous band
    //    // then we can use each of these paths to create a partition for the front context of the overlap within the last index
    //    // this can be appended to the back of the band
    //    //
    //    let (mut bundle, prev_band, _) = {
    //        chain_iter.fold(
    //            (vec![], first_band, None),
    //            |(mut bundle, prev_band, prev_ctx), (_end_bound, overlap)| {
    //                // index context of prefix
    //                let ctx_child = if let Some(link) = overlap.link.as_ref() {
    //                    todo!("implement front context indexing");
    //                    //reader
    //                    //    .contexter::<SplitFront>()
    //                    //    .try_context_path(
    //                    //        link.prefix_path
    //                    //            .get_path()
    //                    //            .unwrap()
    //                    //            .clone()
    //                    //            .into_context_path(),
    //                    //        //node.overlap,
    //                    //    )
    //                    //    .map(|(ctx, _)| ctx)
    //                    None
    //                } else {
    //                    None
    //                };

    //                bundle.push(prev_band);
    //                (
    //                    bundle,
    //                    overlap,
    //                    // join previous and current context into
    //                    prev_ctx
    //                        .map(|prev| {
    //                            ctx_child
    //                                .map(|ctx_child| {
    //                                    trav.read_pattern(vec![prev, ctx_child]).unwrap()
    //                                })
    //                                .or(prev)
    //                        })
    //                        .or(ctx_child),
    //                )
    //            },
    //        )
    //    };
    //    bundle.push(prev_band);
    //    let bundle = bundle
    //        .into_iter()
    //        .map(|overlap| overlap.band.into_pattern())
    //        .collect_vec();
    //    let index = trav.graph_mut().insert_patterns(bundle);
    //    //println!("close result: {:?}", index);
    //    Some(index)
    //}

    //pub fn take_past_context_pattern(
    //    &mut self,
    //    trav: impl HasGraphMut,
    //    start_bound: usize,
    //) -> Option<(usize, Pattern)> {
    //    let mut past = self.take_past(start_bound);
    //    match past.chain.len() {
    //        0 => None,
    //        1 => {
    //            let (end_bound, past) = past.chain.pop_last().unwrap();
    //            Some((end_bound, past.band.into_pattern()))
    //        }
    //        _ => {
    //            let past_key = *past.chain.keys().last().unwrap();
    //            let past_index = past.close(trav).unwrap();
    //            Some((past_key, vec![past_index]))
    //        }
    //    }
    //}
}
