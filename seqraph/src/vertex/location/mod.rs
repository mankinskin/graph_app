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

pub trait TraversalOrder: Wide {
    fn sub_index(&self) -> usize;
    fn cmp(&self, other: impl TraversalOrder) -> Ordering {
        match self.width().cmp(&other.width()) {
            Ordering::Equal => self.sub_index().cmp(&other.sub_index()),
            r => r,
        }
    }
}
impl<T: TraversalOrder> TraversalOrder for &T {
    fn sub_index(&self) -> usize {
        TraversalOrder::sub_index(*self)
    }
}
impl TraversalOrder for ChildLocation {
    fn sub_index(&self) -> usize {
        self.sub_index
    }
}