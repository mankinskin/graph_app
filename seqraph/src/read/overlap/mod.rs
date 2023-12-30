mod band;
mod cache;
mod chain;
mod context;

pub use {
    band::*,
    cache::*,
    chain::*,
};

use super::*;

#[derive(Clone, Debug)]
pub struct OverlapLink {
    pub postfix_path: RolePath<End>, // location of postfix/overlap in first index
    pub prefix_path: MatchEnd<RootedRolePath<Start>>, // location of prefix/overlap in second index
}
#[derive(Clone, Debug)]
pub struct Overlap {
    pub link: Option<OverlapLink>,
    pub band: OverlapBand,
}
impl Overlap {
    pub fn appended<
        'a: 'g,
        'g,
        T: Tokenize,
        D: IndexDirection,
    >(mut self, reader: &mut ReadContext<'_>, end: BandEnd) -> Self {
        self.append(reader, end);
        self
    }
    pub fn append<
        'a: 'g,
        'g,
        T: Tokenize,
        D: IndexDirection,
    >(&mut self, reader: &mut ReadContext<'_>, end: BandEnd) {
        self.band.append(reader, end);
        self.link = None;
    }
}