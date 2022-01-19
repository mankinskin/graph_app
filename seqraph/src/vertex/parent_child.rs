use super::*;
use std::{
    collections::{
        HashMap,
        HashSet,
    },
    fmt::Debug,
    hash::Hasher,
};
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct PatternIndex {
    pub(crate) pattern_id: PatternId,
    pub(crate) sub_index: usize,
}
impl PatternIndex {
    pub fn new(pattern_id: PatternId, sub_index: usize) -> Self {
        Self {
            pattern_id,
            sub_index,
        }
    }
}
/// Storage for parent relationship of a child to a parent
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Parent {
    /// width of the parent
    pub width: TokenPosition,
    /// positions of child in parent patterns
    pub pattern_indices: HashSet<PatternIndex>,
}
impl Parent {
    pub fn new(width: TokenPosition) -> Self {
        Self {
            width,
            pattern_indices: Default::default(),
        }
    }
    pub fn get_width(&self) -> TokenPosition {
        self.width
    }
    pub fn any_pattern_index(&self) -> PatternIndex {
        self.pattern_indices.iter().next().unwrap().clone()
    }
    pub fn add_pattern_index(
        &mut self,
        pattern_id: PatternId,
        sub_index: usize,
    ) {
        self.pattern_indices.insert(PatternIndex { pattern_id, sub_index });
    }
    pub fn remove_pattern_index(
        &mut self,
        pattern_id: PatternId,
        sub_index: usize,
    ) {
        self.pattern_indices.remove(&PatternIndex { pattern_id, sub_index });
    }
    pub fn exists_at_pos(
        &self,
        p: usize,
    ) -> bool {
        self.pattern_indices.iter().any(|i| i.sub_index == p)
    }
    pub fn exists_at_pos_in_pattern(
        &self,
        pattern_id: PatternId,
        sub_index: usize,
    ) -> bool {
        self.pattern_indices.contains(&PatternIndex { pattern_id, sub_index })
    }
    pub fn get_index_at_pos(
        &self,
        p: usize,
    ) -> Option<PatternIndex> {
        self.pattern_indices.iter().find(|i| i.sub_index == p)
            .map(Clone::clone)
    }
    pub fn get_index_at_postfix_of(
        &self,
        v: &VertexData,
    ) -> Option<PatternIndex> {
        self.pattern_indices.iter()
            .find(|i| v.expect_child_pattern(&i.pattern_id).len() == i.sub_index + 1)
            .map(Clone::clone)
    }
    /// filter for pattern indices which occur at start of their patterns
    pub fn filter_pattern_indices_at_prefix(&self) -> impl Iterator<Item = &PatternIndex> {
        self.pattern_indices
            .iter()
            .filter(move |pattern_index| pattern_index.sub_index == 0)
    }
    /// filter for pattern indices which occur at end of given patterns
    pub fn filter_pattern_indices_at_end_in_patterns<'a>(
        &'a self,
        patterns: &'a HashMap<PatternId, Pattern>,
    ) -> impl Iterator<Item = &'a PatternIndex> {
        self.pattern_indices
            .iter()
            .filter(move |pattern_index| {
                pattern_index.sub_index + 1
                    == patterns
                        .get(&pattern_index.pattern_id)
                        .expect("Pattern index not in patterns!")
                        .len()
            })
    }
    // filter for pattern indices which occur in given patterns
    //pub fn filter_pattern_indices_in_patterns<'a>(
    //    &'a self,
    //    patterns: &'a HashMap<PatternId, Pattern>,
    //) -> impl Iterator<Item = &'a (PatternId, usize)> {
    //    self.pattern_indices
    //        .iter()
    //        .filter(move |(pattern_index, sub_index)| {
    //            *sub_index
    //                == patterns
    //                    .get(pattern_index)
    //                    .expect("Pattern index not in patterns!")
    //        })
    //}
}

#[derive(Debug, Eq, Clone, Copy)]
pub struct Child {
    pub index: VertexIndex,   // the child index
    pub width: TokenPosition, // the token width
}
impl Child {
    #[allow(unused)]
    pub(crate) const INVALID: Child = Child { index: 0, width: 0 };
    pub fn new(
        index: impl Indexed,
        width: TokenPosition,
    ) -> Self {
        Self {
            index: index.index(),
            width,
        }
    }
    pub fn get_width(&self) -> TokenPosition {
        self.width
    }
    pub fn get_index(&self) -> VertexIndex {
        self.index
    }
}
impl std::cmp::PartialOrd for Child {
    fn partial_cmp(
        &self,
        other: &Self,
    ) -> Option<std::cmp::Ordering> {
        self.index.partial_cmp(&other.index)
    }
}
impl std::hash::Hash for Child {
    fn hash<H: Hasher>(
        &self,
        h: &mut H,
    ) {
        self.index.hash(h);
    }
}
impl std::cmp::Ord for Child {
    fn cmp(
        &self,
        other: &Self,
    ) -> std::cmp::Ordering {
        self.index.cmp(&other.index)
    }
}
impl PartialEq for Child {
    fn eq(
        &self,
        other: &Self,
    ) -> bool {
        self.index == other.index
    }
}
impl PartialEq<VertexIndex> for Child {
    fn eq(
        &self,
        other: &VertexIndex,
    ) -> bool {
        self.index == *other
    }
}
impl PartialEq<VertexIndex> for &'_ Child {
    fn eq(
        &self,
        other: &VertexIndex,
    ) -> bool {
        self.index == *other
    }
}
impl PartialEq<VertexIndex> for &'_ mut Child {
    fn eq(
        &self,
        other: &VertexIndex,
    ) -> bool {
        self.index == *other
    }
}
impl<T: Into<Child> + Clone> From<&'_ T> for Child {
    fn from(o: &'_ T) -> Self {
        (*o).clone().into()
    }
}
impl From<NewTokenIndex> for Child {
    fn from(o: NewTokenIndex) -> Self {
        Self::new(o.index(), 1)
    }
}
impl IntoIterator for Child {
    type Item = Self;
    type IntoIter = std::iter::Once<Child>;
    fn into_iter(self) -> Self::IntoIter {
        std::iter::once(self)
    }
}

impl Indexed for Child {
    fn index(&self) -> VertexIndex {
        self.index
    }
}
impl Wide for Child {
    fn width(&self) -> usize {
        self.width
    }
}