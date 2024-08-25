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
    pub pattern_id: usize,
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
            id: self.pattern_id,
        }
    }
}

impl IntoPatternLocation for &ChildLocation {
    fn into_pattern_location(self) -> PatternLocation {
        (*self).into_pattern_location()
    }
}

impl crate::graph::vertex::has_vertex_index::HasVertexIndex for ChildLocation {
    fn vertex_index(&self) -> crate::graph::vertex::VertexIndex {
        self.parent.index
    }
}

impl crate::graph::vertex::wide::Wide for ChildLocation {
    fn width(&self) -> usize {
        self.parent.width()
    }
}
