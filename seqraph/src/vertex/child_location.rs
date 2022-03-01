use super::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChildLocation {
    pub(crate) parent: Child,
    pub(crate) pattern_id: PatternId,
    pub(crate) sub_index: usize,
}
impl ChildLocation {
    pub fn new(parent: impl AsChild, pattern_id: PatternId, sub_index: usize) -> Self {
        Self {
            parent: parent.as_child(),
            pattern_id,
            sub_index,
        }
    }
}
pub type ChildPath = Vec<ChildLocation>;

pub trait IntoChildLocation {
    fn into_child_location(self) -> ChildLocation;
}
impl IntoChildLocation for ChildLocation {
    fn into_child_location(self) -> ChildLocation {
        self
    }
}
impl IntoChildLocation for &ChildLocation {
    fn into_child_location(self) -> ChildLocation {
        self.clone()
    }
}
impl IntoPatternLocation for ChildLocation {
    fn into_pattern_location(self) -> PatternLocation {
        PatternLocation {
            parent: self.parent,
            pattern_id: self.pattern_id,
        }
    }
}
impl IntoPatternLocation for &ChildLocation {
    fn into_pattern_location(self) -> PatternLocation {
        PatternLocation {
            parent: self.parent.clone(),
            pattern_id: self.pattern_id.clone(),
        }
    }
}