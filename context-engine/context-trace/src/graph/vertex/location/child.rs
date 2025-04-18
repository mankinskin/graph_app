use std::ops::ControlFlow;

use crate::{
    direction::{
        Left,
        Right,
    },
    graph::vertex::{
        has_vertex_index::HasVertexIndex,
        wide::Wide,
    },
    path::mutators::move_path::leaf::MoveLeaf,
    trace::traversable::{
        TravDir,
        Traversable,
    },
};

use super::{
    super::{
        ChildPatterns,
        child::Child,
        has_vertex_index::ToChild,
        pattern::Pattern,
    },
    PatternId,
    PatternLocation,
    SubLocation,
    pattern::IntoPatternLocation,
};
use crate::direction::pattern::PatternDirection;
#[derive(Clone, Debug, PartialEq, Eq, Copy, Hash)]
pub struct ChildLocation {
    pub parent: Child,
    pub pattern_id: PatternId,
    pub sub_index: usize,
}
impl MoveLeaf<Right> for ChildLocation {
    fn move_leaf<Trav: Traversable>(
        &mut self,
        trav: &Trav,
    ) -> ControlFlow<()> {
        let graph = trav.graph();
        let pattern = graph.expect_pattern_at(*self);
        if let Some(next) =
            TravDir::<Trav>::pattern_index_next(pattern, self.sub_index)
        {
            self.sub_index = next;
            ControlFlow::Continue(())
        } else {
            ControlFlow::Break(())
        }
    }
}
impl MoveLeaf<Left> for ChildLocation {
    fn move_leaf<Trav: Traversable>(
        &mut self,
        trav: &Trav,
    ) -> ControlFlow<()> {
        let graph = trav.graph();
        let pattern = graph.expect_pattern_at(*self);
        if let Some(prev) =
            TravDir::<Trav>::pattern_index_prev(pattern, self.sub_index)
        {
            self.sub_index = prev;
            ControlFlow::Continue(())
        } else {
            ControlFlow::Break(())
        }
    }
}

impl ChildLocation {
    pub fn new(
        parent: impl ToChild,
        pattern_id: PatternId,
        sub_index: usize,
    ) -> Self {
        Self {
            parent: parent.to_child(),
            pattern_id,
            sub_index,
        }
    }
    pub fn get_child_in<'a>(
        &self,
        patterns: &'a ChildPatterns,
    ) -> Option<&'a Child> {
        self.get_pattern_in(patterns)
            .and_then(|p| self.get_child_in_pattern(p))
    }
    pub fn expect_child_in<'a>(
        &self,
        patterns: &'a ChildPatterns,
    ) -> &'a Child {
        self.get_child_in(patterns)
            .expect("Expected Child not present in ChildPatterns!")
    }
    pub fn get_child_in_pattern<'a>(
        &self,
        pattern: &'a Pattern,
    ) -> Option<&'a Child> {
        pattern.get(self.sub_index)
    }
    pub fn expect_child_in_pattern<'a>(
        &self,
        pattern: &'a Pattern,
    ) -> &'a Child {
        self.get_child_in_pattern(pattern)
            .expect("Expected Child not present in ChildPatterns!")
    }
    pub fn get_pattern_in<'a>(
        &self,
        patterns: &'a ChildPatterns,
    ) -> Option<&'a Pattern> {
        patterns.get(&self.pattern_id)
    }
    pub fn expect_pattern_in<'a>(
        &self,
        patterns: &'a ChildPatterns,
    ) -> &'a Pattern {
        self.get_pattern_in(patterns)
            .expect("Expected Pattern not present in ChildPatterns!")
    }
    pub fn to_child_location(
        self,
        sub_index: usize,
    ) -> ChildLocation {
        ChildLocation { sub_index, ..self }
    }
    pub fn to_pattern_location(
        self,
        id: PatternId,
    ) -> PatternLocation {
        PatternLocation {
            parent: self.parent,
            id,
        }
    }
    pub fn to_sub_location(self) -> SubLocation {
        SubLocation {
            pattern_id: self.pattern_id,
            sub_index: self.sub_index,
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
            id: self.pattern_id,
        }
    }
}

impl HasVertexIndex for ChildLocation {
    fn vertex_index(&self) -> crate::graph::vertex::VertexIndex {
        self.parent.index
    }
}

impl Wide for ChildLocation {
    fn width(&self) -> usize {
        self.parent.width()
    }
}
