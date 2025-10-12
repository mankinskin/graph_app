use std::{
    cmp::PartialEq,
    fmt::Debug,
    ops::Range,
};

use crate::graph::vertex::{
    PatternId,
    location::ChildLocation,
    pattern::Pattern,
};

use crate::graph::child::{Child};

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct PatternRangeLocation {
    pub parent: Child,
    pub id: PatternId,
    pub range: Range<usize>,
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct PatternLocation {
    pub parent: Child,
    pub id: PatternId,
}

impl PatternLocation {
    pub fn new(
        parent: Child,
        id: PatternId,
    ) -> Self {
        Self { parent, id }
    }
    pub fn to_child_location(
        &self,
        sub_index: usize,
    ) -> ChildLocation {
        ChildLocation {
            parent: self.parent,
            pattern_id: self.id,
            sub_index,
        }
    }
    pub fn with_range(
        self,
        range: Range<usize>,
    ) -> PatternRangeLocation {
        PatternRangeLocation {
            parent: self.parent,
            id: self.id,
            range,
        }
    }
    //#[allow(unused)]
    //pub fn get_pattern<
    //    'a: 'g,
    //    'g,
    //    T: Tokenize,
    //    Trav: HasGraph<T> + 'a,
    //>(&'a self, trav: &'a Trav) -> Option<&Pattern> {
    //    trav.graph().get_pattern_at(self).ok()
    //}
    //#[allow(unused)]
    //pub fn expect_pattern<
    //    'a: 'g,
    //    'g,
    //    T: Tokenize,
    //    Trav: HasGraph<T> + 'a,
    //>(&'a self, trav: &'a Trav) -> &Pattern {
    //    trav.graph().expect_pattern_at(self)
    //}
    pub fn get_pattern_in<'a>(
        &self,
        patterns: &'a crate::graph::vertex::ChildPatterns,
    ) -> Option<&'a Pattern> {
        patterns.get(&self.id)
    }
    pub fn expect_pattern_in<'a>(
        &self,
        patterns: &'a crate::graph::vertex::ChildPatterns,
    ) -> &'a Pattern {
        self.get_pattern_in(patterns)
            .expect("Expected Pattern not present in ChildPatterns!")
    }
}

pub trait IntoPatternLocation {
    fn into_pattern_location(self) -> PatternLocation;
}

impl<P: IntoPatternLocation + Copy> IntoPatternLocation for &'_ P {
    fn into_pattern_location(self) -> PatternLocation {
        (*self).into_pattern_location()
    }
}
impl IntoPatternLocation for PatternLocation {
    fn into_pattern_location(self) -> PatternLocation {
        self
    }
}
