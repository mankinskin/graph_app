use crate::{
    graph::kind::GraphKind,
    HashMap,
    HashSet,
};
use serde::{
    Deserialize,
    Serialize,
};
use crate::graph::vertex::{
    pattern::Pattern,
    PatternId,
    TokenPosition,
};
use crate::graph::vertex::data::VertexData;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, Serialize, Deserialize)]
pub struct PatternIndex {
    pub pattern_id: PatternId,
    pub sub_index: usize,
}

impl PatternIndex {
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

/// Storage for parent relationship of a child to a parent
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
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
        *self.pattern_indices.iter().next().unwrap()
    }
    pub fn add_pattern_index(
        &mut self,
        pattern_id: PatternId,
        sub_index: usize,
    ) {
        self.pattern_indices.insert(PatternIndex {
            pattern_id,
            sub_index,
        });
    }
    pub fn remove_pattern_index(
        &mut self,
        pattern_id: PatternId,
        sub_index: usize,
    ) {
        self.pattern_indices.remove(&PatternIndex {
            pattern_id,
            sub_index,
        });
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
        self.pattern_indices.contains(&PatternIndex {
            pattern_id,
            sub_index,
        })
    }
    pub fn get_index_at_pos(
        &self,
        p: usize,
    ) -> Option<PatternIndex> {
        self.pattern_indices
            .iter()
            .find(|i| i.sub_index == p)
            .map(Clone::clone)
    }
    pub fn get_index_at_postfix_of(
        &self,
        v: &VertexData<impl GraphKind>,
    ) -> Option<PatternIndex> {
        self.pattern_indices
            .iter()
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
        self.pattern_indices.iter().filter(move |pattern_index| {
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
