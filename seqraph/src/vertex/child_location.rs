use super::*;

use std::cmp::Ordering;

#[derive(Clone, Debug, PartialEq, Eq, Copy, Hash)]
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
    pub fn get_child_in<'a>(&self, patterns: &'a ChildPatterns) -> Option<&'a Child> {
        self.get_pattern_in(patterns)
            .and_then(|p| self.get_child_in_pattern(p))
    }
    pub fn expect_child_in<'a>(&self, patterns: &'a ChildPatterns) -> &'a Child {
        self.get_child_in(patterns).expect("Expected Child not present in ChildPatterns!")
    }
    pub fn get_child_in_pattern<'a>(&self, pattern: &'a Pattern) -> Option<&'a Child> {
        pattern.get(self.sub_index)
    }
    pub fn expect_child_in_pattern<'a>(&self, pattern: &'a Pattern) -> &'a Child {
        self.get_child_in_pattern(pattern).expect("Expected Child not present in ChildPatterns!")
    }
    pub fn get_pattern_in<'a>(&self, patterns: &'a ChildPatterns) -> Option<&'a Pattern> {
        patterns.get(&self.pattern_id)
    }
    pub fn expect_pattern_in<'a>(&self, patterns: &'a ChildPatterns) -> &'a Pattern {
        self.get_pattern_in(patterns).expect("Expected Pattern not present in ChildPatterns!")
    }
    pub fn to_child_location(self, sub_index: usize) -> ChildLocation {
        ChildLocation {
            sub_index,
            ..self
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
impl Wide for ChildLocation {
    fn width(&self) -> usize {
        self.parent.width
    }
}