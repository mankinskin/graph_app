use crate::{
    vertex::*,
};
use std::cmp::PartialEq;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SplitSegment {
    Pattern(Pattern, PatternLocation),
    Child(Child),
}
impl SplitSegment {
    pub fn with_location(p: impl IntoPattern<Item=impl AsChild>, loc: PatternLocation) -> Self {
        Self::Pattern(p.into_pattern(), loc)
    }
    pub fn pattern(self) -> Option<Pattern> {
        match self {
            Self::Child(_) => None,
            Self::Pattern(p, _) => Some(p),
        }
    }
    pub fn child(self) -> Option<Child> {
        match self {
            Self::Pattern(_, _) => None,
            Self::Child(c) => Some(c),
        }
    }
    pub fn is_pattern(&self) -> bool {
        matches!(self, Self::Pattern(_, _))
    }
    pub fn is_child(&self) -> bool {
        matches!(self, Self::Child(_))
    }
    pub fn map_pattern(
        self,
        f: impl FnOnce(Pattern, PatternLocation) -> (Pattern, PatternLocation),
    ) -> Self {
        match self {
            Self::Pattern(p, loc) => {
                let (p, loc) = f(p, loc);
                Self::Pattern(p, loc)
            },
            _ => self,
        }
    }
    pub fn map_child(
        self,
        f: impl FnOnce(Child) -> Child,
    ) -> Self {
        match self {
            Self::Child(c) => Self::Child(f(c)),
            _ => self,
        }
    }
    pub fn unwrap_pattern(self) -> Pattern {
        self.pattern()
            .expect("called SplitSegment::unwrap_pattern on a `Child` value")
    }
    pub fn unwrap_child(self) -> Child {
        self.child()
            .expect("called SplitSegment::unwrap_child on a `Pattern` value")
    }
    pub fn len(&self) -> usize {
        match self {
            Self::Child(_) => 1,
            Self::Pattern(p, _) => {
                let l = p.len();
                assert!(l != 1, "SplitSegment with len = 1 should be a Child!");
                l
            }
        }
    }
    pub fn is_empty(&self) -> bool {
        match self {
            Self::Child(_) => false,
            Self::Pattern(p, _) => p.is_empty(),
        }
    }
}
impl From<Result<Child, (Pattern, PatternLocation)>> for SplitSegment {
    fn from(r: Result<Child, (Pattern, PatternLocation)>) -> Self {
        match r {
            Ok(c) => c.into(),
            Err(p) => p.into(),
        }
    }
}
impl From<Child> for SplitSegment {
    fn from(c: Child) -> Self {
        Self::Child(c)
    }
}
impl From<(Pattern, PatternLocation)> for SplitSegment {
    fn from((p, loc): (Pattern, PatternLocation)) -> Self {
        if p.len() == 1 {
            (*p.first().unwrap()).into()
        } else {
            Self::Pattern(p, loc)
        }
    }
}
impl IntoIterator for SplitSegment {
    type Item = Child;
    type IntoIter = std::vec::IntoIter<Child>;
    fn into_iter(self) -> Self::IntoIter {
        match self {
            Self::Pattern(p, _) => p.into_iter(),
            Self::Child(c) => vec![c].into_iter(),
        }
    }
}
impl IntoPattern for SplitSegment {
    type Token = Child;
    fn as_pattern_view(&'_ self) -> &'_ [Self::Token] {
        match self {
            Self::Child(c) => std::slice::from_ref(c),
            Self::Pattern(p, _) => p.as_slice(),
        }
    }
    fn is_empty(&self) -> bool {
        self.is_empty()
    }
}