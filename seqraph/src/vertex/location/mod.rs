use crate::*;

pub mod child;
pub mod pattern;

pub use child::*;
pub use pattern::*;

#[derive(Clone, Debug, PartialEq, Eq, Copy, Hash)]
pub struct SubLocation {
    pub pattern_id: usize,
    pub sub_index: usize,
}
impl SubLocation {
    pub fn new(pattern_id: PatternId, sub_index: usize) -> Self {
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
        *self
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
            parent: self.parent,
            pattern_id: self.pattern_id,
        }
    }
}
impl Indexed for ChildLocation {
    fn index(&self) -> VertexIndex {
        self.parent.index
    }
}
impl Wide for ChildLocation {
    fn width(&self) -> usize {
        self.parent.width
    }
}

