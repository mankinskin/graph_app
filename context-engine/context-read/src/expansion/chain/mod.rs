pub mod band;
pub mod expand;
pub mod link;

use std::collections::BTreeSet;

use band::Band;
use context_trace::{
    self,
    graph::vertex::{
        child::Child,
        wide::Wide,
    },
};
use derive_more::{
    Deref,
    DerefMut,
};
use tracing::debug;

use crate::expansion::chain::{
    band::BandCtx,
    link::ChainOp,
};

#[derive(Default, Clone, Debug, Deref, DerefMut)]
pub struct BandChain {
    #[deref]
    #[deref_mut]
    pub bands: BTreeSet<Band>,
    // todo: use map for links
    //pub links: VecDeque<OverlapLink>,
}
impl BandChain {
    pub fn new(index: Child) -> Self {
        debug!("New BandChain");
        Self {
            bands: Some(Band {
                pattern: vec![index],
                start_bound: 0,
                end_bound: index.width(),
            })
            .into_iter()
            .collect(),
            //links: Default::default(),
        }
    }
    pub fn ends_at(
        &self,
        bound: usize,
    ) -> Option<BandCtx<'_>> {
        debug!("ends_at");
        let band = self.bands.get(&bound)?;
        debug!("Does end at {:?}", bound);

        Some(BandCtx {
            band,
            //back_link: self.links.iter().last(),
            //front_link: None,
        })
    }
    pub fn last(&self) -> Option<BandCtx<'_>> {
        self.bands.iter().last().map(|band| BandCtx {
            band,
            //back_link: self.links.iter().last(),
            //front_link: None,
        })
    }
    pub fn append(
        &mut self,
        band: impl Into<Band>,
    ) {
        self.bands.insert(band.into());
    }
    pub fn append_front_complement(
        &mut self,
        complement: Child,
        exp: Child,
    ) {
        debug!("append_front_complement");
        let pattern = vec![complement, exp];
        let band = Band::from((0, pattern));
        self.append(band);
    }
    pub fn pop_first(&mut self) -> Option<Band> {
        //self.links.pop_front();
        self.bands.pop_first()
    }
}
