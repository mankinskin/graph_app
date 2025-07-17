pub mod band;
pub mod expand;
pub mod link;

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
        accessors::role::End,
        structs::role_path::RolePath,
    },
};
use derive_more::From;

use crate::expansion::chain::{
    band::BandCtx,
    link::{
        ChainAppendage,
        OverlapLink,
    },
};

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

#[derive(Default, Clone, Debug)]
pub struct BandChain {
    pub bands: BTreeSet<Band>,
    pub links: VecDeque<OverlapLink>,
}
impl BandChain {
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
}
