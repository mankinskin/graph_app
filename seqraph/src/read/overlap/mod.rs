use band::{
    BandEnd,
    OverlapBand,
};

use crate::read::{
    bundle::band,
    reader::context::ReadContext,
};
use hypercontext_api::path::{
    accessors::role::{
        End,
        Start,
    },
    structs::{
        match_end::MatchEnd,
        role_path::RolePath,
        rooted::{
            role_path::RootedRolePath,
            root::IndexRoot,
        },
    },
};
pub mod cache;
pub mod chain;
pub mod primer;

#[derive(Clone, Debug)]
pub struct OverlapLink {
    pub postfix_path: RolePath<End>, // location of postfix/overlap in first index
    pub prefix_path: MatchEnd<RootedRolePath<Start, IndexRoot>>, // location of prefix/overlap in second index
}

#[derive(Clone, Debug)]
pub struct Overlap {
    pub link: Option<OverlapLink>,
    pub band: OverlapBand,
}

impl Overlap {
    pub fn appended(
        mut self,
        end: BandEnd,
    ) -> Self {
        self.append(end);
        self
    }
    pub fn append(
        &mut self,
        end: BandEnd,
    ) {
        self.band.append(end);
        self.link = None;
    }
}
