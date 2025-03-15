use std::num::NonZeroUsize;

use crate::graph::vertex::{
    child::Child,
    wide::Wide,
};

use std::fmt::Debug;

pub mod vertex;

pub mod leaves;
pub mod position;

#[derive(Clone, Debug, Copy, Hash, Eq, PartialEq)]
pub struct PosKey {
    pub index: Child,
    pub pos: NonZeroUsize,
}

impl PosKey {
    pub fn new<P: TryInto<NonZeroUsize>>(
        index: Child,
        pos: P,
    ) -> Self
    where
        P::Error: Debug,
    {
        Self {
            index,
            pos: pos.try_into().unwrap(),
        }
    }
}

impl From<Child> for PosKey {
    fn from(index: Child) -> Self {
        Self {
            index,
            pos: NonZeroUsize::new(index.width()).unwrap(),
        }
    }
}
