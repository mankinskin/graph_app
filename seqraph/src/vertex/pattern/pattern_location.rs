use super::*;

#[derive(Clone, Debug, PartialEq, Eq, Copy)]
pub struct PatternLocation {
    pub parent: Child,
    pub pattern_id: PatternId,
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
    #[allow(unused)]
    pub fn get_pattern<
        'a: 'g,
        'g,
        T: Tokenize,
        Trav: Traversable<T>,
    >(&self, trav: &'a Trav) -> Option<Pattern> {
        trav.graph().get_pattern_at(self).ok()
    }
    #[allow(unused)]
    pub fn expect_pattern<
        'a: 'g,
        'g,
        T: Tokenize,
        Trav: Traversable<T>,
    >(&self, trav: &'a Trav) -> Pattern {
        trav.graph().expect_pattern_at(self)
    }
    pub fn get_pattern_in<'a>(&self, patterns: &'a ChildPatterns) -> Option<&'a Pattern> {
        patterns.get(&self.pattern_id)
    }
    pub fn expect_pattern_in<'a>(&self, patterns: &'a ChildPatterns) -> &'a Pattern {
        self.get_pattern_in(patterns).expect("Expected Pattern not present in ChildPatterns!")
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