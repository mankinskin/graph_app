use super::{
    super::{
        child::Child,
        indexed::AsChild,
        pattern::Pattern,
        ChildPatterns,
    },
    PatternId,
    PatternLocation,
    SubLocation,
};

#[derive(Clone, Debug, PartialEq, Eq, Copy, Hash)]
pub struct ChildLocation {
    pub parent: Child,
    pub pattern_id: PatternId,
    pub sub_index: usize,
}
impl ChildLocation {
    pub fn new(
        parent: impl AsChild,
        pattern_id: PatternId,
        sub_index: usize,
    ) -> Self {
        Self {
            parent: parent.as_child(),
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
        id: usize,
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
