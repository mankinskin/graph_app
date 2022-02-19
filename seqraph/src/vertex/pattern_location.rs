use crate::{
    vertex::*,
};
use std::ops::Range;

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct PatternRangeLocation {
    pub parent: Child,
    pub pattern_id: PatternId,
    pub range: Range<usize>,
}
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct PatternLocation {
    pub parent: Child,
    pub pattern_id: PatternId,
}
impl PatternLocation {
    pub fn new(parent: Child, pattern_id: PatternId) -> Self {
        Self {
            parent,
            pattern_id,
        }
    }
    pub fn with_range(self, range: Range<usize>) -> PatternRangeLocation {
        PatternRangeLocation {
            parent: self.parent,
            pattern_id: self.pattern_id,
            range,
        }
    }
}