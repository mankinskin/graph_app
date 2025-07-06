use context_trace::{
    self,
    path::{
        accessors::role::{
            End,
            Start,
        },
        structs::role_path::RolePath,
    },
};

use crate::expansion::chain::{
    band::Band,
    BandChain,
};

pub trait ChainAppendage: Sized {
    fn append_to_chain(
        self,
        chain: &mut BandChain,
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
#[derive(Clone, Debug)]
pub struct OverlapLink {
    pub postfix_path: RolePath<End>, // location of postfix/overlap in first index
    pub prefix_path: RolePath<Start>, // location of prefix/overlap in second index
}
