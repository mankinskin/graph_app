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
        has_vertex_index::ToChild,
        wide::Wide,
    },
    path::structs::rooted::{
        pattern_range::PatternRangePath,
        role_path::IndexEndPath,
    },
};
use derive_more::From;
use derive_new::new;

use crate::{
    context::ReadCtx,
    expansion::chain::{
        band::BandCtx,
        expand::ExpandCtx,
        link::OverlapLink,
    },
};

#[derive(Debug, From)]
pub enum ChainOp {
    Expansion(BandExpansion),
    Cap(BandCap),
}
#[derive(Debug)]
pub struct BandExpansion {
    pub expansion: IndexWithPath,
    pub start_bound: usize,
    pub postfix_path: IndexEndPath,
}
impl StartBound for BandExpansion {
    fn start_bound(&self) -> usize {
        self.start_bound
    }
}
#[derive(Debug)]
pub struct BandCap {
    pub postfix_path: IndexEndPath,
    pub expansion: Child,
    pub start_bound: usize,
}

pub trait StartBound: Sized {
    fn start_bound(&self) -> usize;
}
impl StartBound for (usize, Band) {
    fn start_bound(&self) -> usize {
        self.0
    }
}

#[derive(Clone, Debug)]
pub struct BoundedBand {
    pub band: Band,
    pub end_bound: usize,
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
        band: impl Into<Band>,
    ) {
        self.bands.insert(band.into());
    }
    pub fn append_front_complement(
        &mut self,
        complement: Child,
        exp: impl ToChild,
    ) {
        let index = exp.to_child();
        let pattern = vec![complement, index];
        let band = Band::from((0, pattern));
        self.append(band);
    }
    pub fn pop_first(&mut self) -> Option<Band> {
        self.links.pop_front();
        self.bands.pop_first()
    }
}

#[derive(Debug, new)]
pub struct ChainCtx<'a> {
    pub trav: ReadCtx,
    pub cursor: &'a mut PatternRangePath,
    pub chain: BandChain,
}
impl ChainCtx<'_> {
    pub fn last(&self) -> &Band {
        self.chain.last().unwrap().band
    }
}
impl Iterator for ChainCtx<'_> {
    type Item = ChainOp;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(mut ctx) = ExpandCtx::try_new(self) {
            ctx.find_map(|op| match &op {
                ChainOp::Expansion(_) => Some(op),
                ChainOp::Cap(cap) =>
                    self.chain.ends_at(cap.start_bound).map(|_| op),
            })
        } else {
            None
        }
    }
}
