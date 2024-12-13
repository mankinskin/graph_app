use hypercontext_api::graph::vertex::{
    child::Child,
    wide::Wide,
};
use crate::read::{
    bands::{
        band::{OverlapBand, OverlapBundle},
        overlaps::overlap::{chain::OverlapChain, Overlap}
    },
    reader::context::ReadContext,
};

#[derive(Default, Clone, Debug)]
pub struct OverlapCache {
    pub chain: OverlapChain,
    pub end_bound: usize,
    pub last: Option<Overlap>,
}

impl OverlapCache {
    pub fn new(first: Child) -> Self {
        Self {
            end_bound: first.width(),
            last: Overlap {
                link: None,
                band: OverlapBand::from(first),
            }
            .into(),
            chain: OverlapChain::default(),
        }
    }
    pub fn add_bundle(
        &mut self,
        reader: &mut ReadContext<'_>,
        bundle: OverlapBundle,
    ) {
        self.chain.path.insert(
            self.end_bound,
            Overlap {
                link: None,
                band: bundle.into_band(reader),
            },
        );
    }
    pub fn append(
        &mut self,
        _reader: &mut ReadContext<'_>,
        start_bound: usize,
        overlap: Overlap,
    ) {
        let width = overlap.band.end.index().unwrap().width();
        if let Some(last) = self.last.replace(overlap) {
            self.chain.add_overlap(self.end_bound, last).unwrap()
        }
        self.end_bound = start_bound + width;
    }
}
