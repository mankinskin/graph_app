use derive_builder::Builder;
use either::Either;
use itertools::Itertools;
use serde::{
    Deserialize,
    Serialize,
};
use std::{
    fmt::{
        Debug,
        Display,
    },
    num::NonZeroUsize,
    slice::SliceIndex,
};

use crate::{
    direction::{
        pattern::PatternDirection,
        Direction,
    },
    graph::{
        getters::ErrorReason,
        kind::GraphKind,
        vertex::{
            child::Child,
            has_vertex_index::{
                HasVertexIndex,
                ToChild,
            },
            key::VertexKey,
            location::{
                child::ChildLocation,
                SubLocation,
            },
            parent::{
                Parent,
                PatternIndex,
            },
            pattern::{
                self,
                pattern_range::PatternRangeIndex,
                pattern_width,
                IntoPattern,
                Pattern,
            },
            wide::Wide,
            ChildPatterns,
            IndexPosition,
            PatternId,
            VertexIndex,
            VertexParents,
        },
        Hypergraph,
    },
    traversal::{
        iterator::bands::{
            BandIterator,
            PostfixIterator,
            PrefixIterator,
        },
        traversable::{
            TravDir,
            Traversable,
        },
    },
    HashSet,
};

pub fn clone_child_patterns(children: &'_ ChildPatterns) -> impl Iterator<Item = Pattern> + '_ {
    children.iter().map(|(_, p)| p.clone())
}
pub fn localized_children_iter_for_index(
    parent: impl ToChild,
    children: &ChildPatterns,
) -> impl IntoIterator<Item = (ChildLocation, &Child)> {
    let parent = parent.to_child();
    children.iter().flat_map(move |(&pid, pat)| {
        pat.iter()
            .enumerate()
            .map(move |(i, c)| (ChildLocation::new(parent, pid, i), c))
    })
}

#[derive(Debug, PartialEq, Eq, Clone, Builder, Serialize, Deserialize)]
pub struct VertexData {
    pub width: usize,
    pub index: VertexIndex,

    #[builder(default)]
    pub key: VertexKey,

    #[builder(default)]
    pub parents: VertexParents,

    #[builder(default)]
    pub children: ChildPatterns,
}

impl VertexData {
    pub fn new(
        index: VertexIndex,
        width: usize,
    ) -> Self {
        Self {
            width,
            key: VertexKey::default(),
            index,
            parents: VertexParents::default(),
            children: ChildPatterns::default(),
        }
    }
    pub fn get_width(&self) -> usize {
        self.width
    }
    pub fn postfix_iter<'a, Trav: Traversable + 'a>(
        &self,
        trav: Trav,
    ) -> PostfixIterator<'a, Trav>
    where
        <TravDir<Trav> as Direction>::Opposite: PatternDirection,
    {
        PostfixIterator::band_iter(trav, self.to_child())
    }
    pub fn prefix_iter<'a, Trav: Traversable + 'a>(
        &self,
        trav: Trav,
    ) -> PrefixIterator<'a, Trav> {
        PrefixIterator::band_iter(trav, self.to_child())
    }
    pub fn get_parent(
        &self,
        index: impl HasVertexIndex,
    ) -> Result<&Parent, ErrorReason> {
        let index = index.vertex_index();
        self.parents
            .get(&index)
            .ok_or(ErrorReason::ErrorReasoningParent(index))
    }
    pub fn get_parent_mut(
        &mut self,
        index: impl HasVertexIndex,
    ) -> Result<&mut Parent, ErrorReason> {
        let index = index.vertex_index();
        self.parents
            .get_mut(&index)
            .ok_or(ErrorReason::ErrorReasoningParent(index))
    }
    #[track_caller]
    pub fn expect_parent(
        &self,
        index: impl HasVertexIndex,
    ) -> &Parent {
        self.get_parent(index).unwrap()
    }
    #[track_caller]
    pub fn expect_parent_mut(
        &mut self,
        index: impl HasVertexIndex,
    ) -> &mut Parent {
        self.get_parent_mut(index).unwrap()
    }
    pub fn get_parents(&self) -> &VertexParents {
        &self.parents
    }
    pub fn get_parents_mut(&mut self) -> &mut VertexParents {
        &mut self.parents
    }
    pub fn get_child_pattern_range<R: PatternRangeIndex>(
        &self,
        id: &PatternId,
        range: R,
    ) -> Result<&<R as SliceIndex<[Child]>>::Output, ErrorReason> {
        self.get_child_pattern(id)
            .and_then(|p| pattern::pattern_range::get_child_pattern_range(id, p, range.clone()))
    }
    #[track_caller]
    pub fn expect_child_pattern_range<R: PatternRangeIndex>(
        &self,
        id: &PatternId,
        range: R,
    ) -> &<R as SliceIndex<[Child]>>::Output {
        let p = self.expect_child_pattern(id);
        pattern::pattern_range::get_child_pattern_range(id, p, range.clone())
            .expect("Range in pattern")
    }
    pub fn get_child_pattern_position(
        &self,
        id: &PatternId,
        pos: IndexPosition,
    ) -> Result<&Child, ErrorReason> {
        self.children
            .get(id)
            .and_then(|p| p.get(pos))
            .ok_or(ErrorReason::NoChildPatterns)
    }
    pub fn get_child_pattern_with_prefix_width(
        &self,
        width: NonZeroUsize,
    ) -> Option<(&PatternId, &Pattern)> {
        self.children
            .iter()
            .find(|(_pid, pat)| pat[0].width() == width.get())
    }
    pub fn get_child_pattern(
        &self,
        id: &PatternId,
    ) -> Result<&Pattern, ErrorReason> {
        self.children
            .get(id)
            .ok_or(ErrorReason::InvalidPattern(*id))
    }
    pub fn get_child_at(
        &self,
        location: &SubLocation,
    ) -> Result<&Child, ErrorReason> {
        self.children
            .get(&location.pattern_id)
            .ok_or(ErrorReason::InvalidPattern(location.pattern_id))?
            .get(location.sub_index)
            .ok_or(ErrorReason::InvalidChild(location.sub_index))
    }
    pub fn expect_child_at(
        &self,
        location: &SubLocation,
    ) -> &Child {
        self.get_child_at(location).unwrap()
    }
    pub fn get_child_mut_at(
        &mut self,
        location: &SubLocation,
    ) -> Result<&mut Child, ErrorReason> {
        self.children
            .get_mut(&location.pattern_id)
            .ok_or(ErrorReason::InvalidPattern(location.pattern_id))?
            .get_mut(location.sub_index)
            .ok_or(ErrorReason::InvalidChild(location.sub_index))
    }
    pub fn expect_child_mut_at(
        &mut self,
        location: &SubLocation,
    ) -> &mut Child {
        self.get_child_mut_at(location).unwrap()
    }
    #[track_caller]
    pub fn expect_pattern_len(
        &self,
        id: &PatternId,
    ) -> usize {
        self.expect_child_pattern(id).len()
    }
    pub fn expect_child_offset(
        &self,
        loc: &SubLocation,
    ) -> usize {
        pattern_width(&self.expect_child_pattern(&loc.pattern_id)[0..loc.sub_index])
    }
    pub fn find_child_pattern_id(
        &self,
        f: impl FnMut(&(&PatternId, &Pattern)) -> bool,
    ) -> Option<PatternId> {
        self.children.iter().find(f).map(|r| *r.0)
    }
    pub fn get_child_pattern_mut(
        &mut self,
        id: &PatternId,
    ) -> Result<&mut Pattern, ErrorReason> {
        self.children
            .get_mut(id)
            .ok_or(ErrorReason::NoChildPatterns)
    }
    #[track_caller]
    pub fn expect_any_child_pattern(&self) -> (&PatternId, &Pattern) {
        self.children
            .iter()
            .next()
            .unwrap_or_else(|| panic!("Pattern vertex has no children {:#?}", self,))
    }
    #[track_caller]
    pub fn expect_child_pattern(
        &self,
        id: &PatternId,
    ) -> &Pattern {
        self.get_child_pattern(id).unwrap_or_else(|_| {
            panic!(
                "Child pattern with id {} does not exist in in vertex {:#?}",
                id, self,
            )
        })
    }
    #[track_caller]
    pub fn expect_child_pattern_mut(
        &mut self,
        id: &PatternId,
    ) -> &mut Pattern {
        self.get_child_pattern_mut(id)
            .unwrap_or_else(|_| panic!("Child pattern with id {} does not exist in in vertex", id,))
    }
    pub fn get_child_patterns(&self) -> &ChildPatterns {
        &self.children
    }
    pub fn get_child_patterns_mut(&mut self) -> &mut ChildPatterns {
        &mut self.children
    }
    pub fn get_child_pattern_iter(&'_ self) -> impl Iterator<Item = Pattern> + '_ {
        clone_child_patterns(&self.children)
    }
    pub fn get_child_pattern_set(&self) -> HashSet<Pattern> {
        self.get_child_pattern_iter().collect()
    }
    pub fn get_child_pattern_vec(&self) -> Vec<Pattern> {
        self.get_child_pattern_iter().collect()
    }
    pub fn add_pattern_no_update(
        &mut self,
        id: PatternId,
        pat: impl IntoPattern,
    ) {
        if pat.borrow().len() < 2 {
            assert!(pat.borrow().len() > 1);
        }
        self.children.insert(id, pat.into_pattern());
        self.validate();
    }
    pub fn add_patterns_no_update(
        &mut self,
        patterns: impl IntoIterator<Item = (PatternId, impl IntoPattern)>,
    ) {
        for (id, pat) in patterns {
            if pat.borrow().len() < 2 {
                assert!(pat.borrow().len() > 1);
            }
            self.children.insert(id, pat.into_pattern());
        }
        self.validate();
    }
    #[track_caller]
    pub fn validate_links(&self) {
        assert!(self.children.len() != 1 || self.parents.len() != 1);
    }
    #[track_caller]
    pub fn validate_patterns(&self) {
        self.children
            .iter()
            .fold(Vec::new(), |mut acc: Vec<Vec<usize>>, (_pid, p)| {
                let mut offset = 0;
                assert!(!p.is_empty());
                let mut p = p.iter().fold(Vec::new(), |mut pa, c| {
                    offset += c.width();
                    assert!(
                        !acc.iter().any(|pr| pr.contains(&offset)),
                        "Duplicate border in index child patterns"
                    );
                    pa.push(offset);
                    pa
                });
                p.pop().expect("Empty pattern!");
                assert!(!p.is_empty(), "Single index pattern");
                assert_eq!(offset, self.width);
                acc.push(p);
                acc
            });
    }
    #[track_caller]
    pub fn validate(&self) {
        //self.validate_links();
        if !self.children.is_empty() {
            self.validate_patterns();
        }
    }
    pub fn add_parent(
        &mut self,
        loc: ChildLocation,
    ) {
        if let Some(parent) = self.parents.get_mut(&loc.parent.vertex_index()) {
            parent.add_pattern_index(loc.pattern_id, loc.sub_index);
        } else {
            let mut parent_rel = Parent::new(loc.parent.width());
            parent_rel.add_pattern_index(loc.pattern_id, loc.sub_index);
            self.parents.insert(loc.parent.vertex_index(), parent_rel);
        }
        // not while indexing
        //self.validate_links();
    }
    pub fn remove_parent(
        &mut self,
        vertex: impl HasVertexIndex,
    ) {
        self.parents.remove(&vertex.vertex_index());
        // not while indexing
        //self.validate_links();
    }
    pub fn remove_parent_index(
        &mut self,
        vertex: impl HasVertexIndex,
        pattern: PatternId,
        index: usize,
    ) {
        if let Some(parent) = self.parents.get_mut(&vertex.vertex_index()) {
            if parent.pattern_indices.len() > 1 {
                parent.remove_pattern_index(pattern, index);
            } else {
                self.parents.remove(&vertex.vertex_index());
            }
        }
        // not while indexing
        //self.validate_links();
    }
    pub fn get_parents_below_width(
        &self,
        width_ceiling: Option<usize>,
    ) -> impl Iterator<Item = (&VertexIndex, &Parent)> + Clone {
        let parents = self.get_parents();
        // optionally filter parents by width
        if let Some(ceil) = width_ceiling {
            Either::Left(
                parents
                    .iter()
                    .filter(move |(_, parent)| parent.get_width() < ceil),
            )
        } else {
            Either::Right(parents.iter())
        }
    }
    pub fn to_pattern_strings<G: GraphKind>(
        &self,
        g: &Hypergraph<G>,
    ) -> Vec<Vec<String>>
    where
        G::Token: Display,
    {
        self.get_child_pattern_iter()
            .map(|pat| {
                pat.iter()
                    .map(|c| g.index_string(c.index))
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>()
    }
    //pub fn get_parents_at_prefix(&self) -> HashMap<VertexIndex, PatternId> {
    //    self.get_parents_with_index_at(0)
    //}
    //pub fn get_parents_at_postfix(
    //    &self,
    //    graph: &crate::graph::Hypergraph,
    //) -> HashMap<VertexIndex, PatternId> {
    //    self.parents
    //        .iter()
    //        .filter_map(|(id, parent)| {
    //            parent
    //                .get_index_at_postfix_of(graph.expect_vertex(id))
    //                .map(|pat| (*id, pat.pattern_id))
    //        })
    //        .collect()
    //}
    //pub fn get_parents_with_index_at(
    //    &self,
    //    offset: usize,
    //) -> HashMap<VertexIndex, PatternId> {
    //    self.parents
    //        .iter()
    //        .filter_map(|(id, parent)| {
    //            parent
    //                .get_index_at_pos(offset)
    //                .map(|pat| (*id, pat.pattern_id))
    //        })
    //        .collect()
    //}
    //pub fn filter_parent_to(
    //    &self,
    //    parent: impl HasVertexIndex,
    //    cond: impl Fn(&&Parent) -> bool,
    //) -> Result<&'_ Parent, ErrorReason> {
    //    let index = parent.vertex_index();
    //    self.get_parent(index)
    //        .ok()
    //        .filter(cond)
    //        .ok_or(ErrorReason::ErrorReasoningParent(index))
    //}
    //pub fn get_parent_to_ending_at(
    //    &self,
    //    parent_key: impl HasVertexKey,
    //    offset: usize,
    //) -> Result<&'_ Parent, ErrorReason> {
    //    self.filter_parent_to(parent_key, |parent| {
    //        offset
    //            .checked_sub(self.width)
    //            .map(|p| parent.exists_at_pos(p))
    //            .unwrap_or(false)
    //    })
    //}
    pub fn get_parent_to_starting_at(
        &self,
        parent_index: impl HasVertexIndex,
        index_offset: usize,
    ) -> Result<PatternIndex, ErrorReason> {
        let index = parent_index.vertex_index();
        self.get_parent(index)
            .ok()
            .and_then(|parent| parent.get_index_at_pos(index_offset))
            .ok_or(ErrorReason::ErrorReasoningParent(index))
    }
    pub fn get_parent_at_prefix_of(
        &self,
        index: impl HasVertexIndex,
    ) -> Result<PatternIndex, ErrorReason> {
        self.get_parent_to_starting_at(index, 0)
    }
    pub fn get_parent_at_postfix_of(
        &self,
        vertex: &VertexData,
    ) -> Result<PatternIndex, ErrorReason> {
        self.get_parent(vertex.vertex_index())
            .ok()
            .and_then(|parent| parent.get_index_at_postfix_of(vertex))
            .ok_or(ErrorReason::ErrorReasoningParent(vertex.vertex_index()))
    }
    //pub fn find_ancestor_with_range(
    //    &self,
    //    half: Pattern,
    //    range: impl PatternRangeIndex,
    //) -> Result<PatternId, ErrorReason> {
    //    self.children
    //        .iter()
    //        .find_map(|(id, pat)| {
    //            if pat[range.clone()] == half[..] {
    //                Some(*id)
    //            } else {
    //                None
    //            }
    //        })
    //        .ok_or(ErrorReason::NoChildPatterns)
    //}
    pub fn largest_postfix(&self) -> (PatternId, Child) {
        let (id, c) = self
            .children
            .iter()
            .fold(None, |acc: Option<(&PatternId, &Child)>, (pid, p)| {
                if let Some(acc) = acc {
                    let c = p.last().unwrap();
                    if c.width() > acc.1.width() {
                        Some((pid, c))
                    } else {
                        Some(acc)
                    }
                } else {
                    Some((pid, p.last().unwrap()))
                }
            })
            .unwrap();
        (*id, *c)
    }
    pub fn all_children_iter(&self) -> impl IntoIterator<Item = &Child> {
        self.children.iter().flat_map(|(_, pat)| pat.iter())
    }
    pub fn all_localized_children_iter(&self) -> impl IntoIterator<Item = (ChildLocation, &Child)> {
        localized_children_iter_for_index(self.to_child(), &self.children)
    }
    pub fn top_down_containment_nodes(&self) -> Vec<(usize, Child)> {
        self.children
            .iter()
            .flat_map(|(_, pat)| {
                pat.iter()
                    .enumerate()
                    .filter(|(_, c)| c.width() + 1 == self.width())
                    .map(|(off, c)| (off, *c))
            })
            .sorted_by_key(|&(off, _)| off)
            .collect_vec()
    }
}
