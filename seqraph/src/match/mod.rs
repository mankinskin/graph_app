use crate::{
    search::*,
    vertex::*,
    *,
};
mod matcher;
pub use matcher::*;
mod match_direction;
pub use match_direction::*;
//mod async_matcher;
//pub use async_matcher::*;
//mod async_match_direction;
//pub use async_match_direction::*;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum NoMatch {
    EmptyPatterns,
    NoParents,
    Mismatch(MismatchPath),
    NoChildPatterns,
    NotFound(Pattern),
    NoMatchingParent(VertexIndex),
    InvalidPattern(PatternId),
    InvalidPatternRange(PatternId, Pattern, String),
    SingleIndex,
    ParentMatchingPartially,
    UnknownKey,
    UnknownIndex,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GrownPath {
    pub(crate) path: ChildPath,
    pub(crate) remainder: GrowthRemainder,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GrowthRemainder {
    Query(Pattern),
    Child(Pattern),
    None,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MismatchPath {
    pub path: ChildPath,
    pub child: Pattern,
    pub query: Pattern,
}
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

impl<'t, 'a, T> Hypergraph<T>
where
    T: Tokenize + 't,
{
    pub fn right_matcher(&'a self) -> Matcher<'a, T, Right> {
        Matcher::new(self)
    }
    pub fn left_matcher(&'a self) -> Matcher<'a, T, Left> {
        Matcher::new(self)
    }
}