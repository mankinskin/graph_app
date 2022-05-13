use super::*;

#[derive(Clone, Debug, PartialEq, Eq, Copy)]
pub struct PatternLocation {
    pub(crate) parent: Child,
    pub(crate) pattern_id: PatternId,
}
impl PatternLocation {
    pub fn new(parent: impl AsChild, pattern_id: PatternId) -> Self {
        Self {
            parent: parent.as_child(),
            pattern_id,
        }
    }
    pub fn to_child_location(&self, sub_index: usize) -> ChildLocation {
        ChildLocation {
            parent: self.parent,
            pattern_id: self.pattern_id,
            sub_index,
        }
    }
}

pub trait IntoPatternLocation {
    fn into_pattern_location(self) -> PatternLocation;
}
impl IntoPatternLocation for PatternLocation {
    fn into_pattern_location(self) -> PatternLocation {
        self
    }
}
impl IntoPatternLocation for &PatternLocation {
    fn into_pattern_location(self) -> PatternLocation {
        *self
    }
}