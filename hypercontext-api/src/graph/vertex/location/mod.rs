use std::{
    cmp::PartialEq,
    fmt::Debug,
};

use child::*;
use pattern::*;

use super::PatternId;

pub mod child;
pub mod pattern;

#[derive(Clone, Debug, PartialEq, Eq, Copy, Hash)]
pub struct SubLocation {
    pub pattern_id: PatternId,
    pub sub_index: usize,
}

impl SubLocation {
    pub fn new(
        pattern_id: PatternId,
        sub_index: usize,
    ) -> Self {
        Self {
            pattern_id,
            sub_index,
        }
    }
}

impl From<ChildLocation> for SubLocation {
    fn from(value: ChildLocation) -> Self {
        value.to_sub_location()
    }
}
