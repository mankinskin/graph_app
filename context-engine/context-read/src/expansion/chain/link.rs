use context_trace::*;
use derive_more::From;

use crate::expansion::chain::band::Band;

#[derive(Clone, Debug)]
pub struct OverlapLink {
    pub postfix_path: RolePath<End>, // location of postfix/overlap in first index
    pub prefix_path: RolePath<Start>, // location of prefix/overlap in second index
}

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
pub trait EndBound: Sized {
    fn end_bound(&self) -> usize;
}
impl StartBound for (usize, Band) {
    fn start_bound(&self) -> usize {
        self.0
    }
}
impl EndBound for (Band, usize) {
    fn end_bound(&self) -> usize {
        self.1
    }
}
