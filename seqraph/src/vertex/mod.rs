use crate::{
    graph::*,
    r#match::*,
    read::*,
};
use either::Either;
use std::{
    collections::{
        HashMap,
        HashSet,
    },
    fmt::Debug,
    slice::SliceIndex,
    sync::atomic::{
        AtomicUsize,
        Ordering,
    },
};

mod indexed;
mod parent_child;
mod pattern;
mod pattern_stream;
mod token;
pub use {
    indexed::*,
    parent_child::*,
    pattern::*,
    pattern_stream::*,
    token::*,
};

pub type VertexIndex = usize;
pub type VertexParents = HashMap<VertexIndex, Parent>;
pub type ChildPatterns = HashMap<PatternId, Pattern>;
pub type PatternId = usize;
pub type TokenPosition = usize;
pub type IndexPosition = usize;
pub type IndexPattern = Vec<VertexIndex>;
pub type VertexPatternView<'a> = Vec<&'a VertexData>;

pub(crate) fn clone_child_patterns<'a>(children: &'a ChildPatterns) -> impl IntoIterator<Item=Pattern> + 'a {
    children.iter().map(|(_, p)| p.clone())
}
#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub enum VertexKey<T: Tokenize> {
    Token(Token<T>),
    Pattern(VertexIndex),
}
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct VertexData {
    pub index: VertexIndex,
    pub width: TokenPosition,
    pub parents: VertexParents,
    pub children: ChildPatterns,
}
impl VertexData {
    fn next_child_pattern_id() -> PatternId {
        static PATTERN_ID_COUNTER: AtomicUsize = AtomicUsize::new(0);
        PATTERN_ID_COUNTER.fetch_add(1, Ordering::Relaxed)
    }
    pub fn new(
        index: VertexIndex,
        width: TokenPosition,
    ) -> Self {
        Self {
            index,
            width,
            parents: VertexParents::new(),
            children: ChildPatterns::new(),
        }
    }
    pub fn get_width(&self) -> TokenPosition {
        self.width
    }
    pub fn get_parent(
        &self,
        index: impl Indexed,
    ) -> Result<&Parent, NoMatch> {
        let index = index.index();
        self.parents
            .get(&index)
            .ok_or(NoMatch::NoMatchingParent(index))
    }
    pub fn get_parent_mut(
        &mut self,
        index: impl Indexed,
    ) -> Result<&mut Parent, NoMatch> {
        let index = index.index();
        self.parents
            .get_mut(&index)
            .ok_or(NoMatch::NoMatchingParent(index))
    }
    pub fn expect_parent(
        &self,
        index: impl Indexed,
    ) -> &Parent {
        self.get_parent(index).unwrap()
    }
    pub fn expect_parent_mut(
        &mut self,
        index: impl Indexed,
    ) -> &mut Parent {
        self.get_parent_mut(index).unwrap()
    }
    pub fn get_parents(&self) -> &VertexParents {
        &self.parents
    }
    pub fn get_parents_mut(&mut self) -> &mut VertexParents {
        &mut self.parents
    }
    pub fn get_child_pattern_range<R: SliceIndex<[Child]>>(
        &self,
        id: &PatternId,
        range: R,
    ) -> Result<&<R as SliceIndex<[Child]>>::Output, NoMatch> {
        self.children
            .get(id)
            .and_then(|p| p.get(range))
            .ok_or(NoMatch::NoChildPatterns)
    }
    pub fn get_child_pattern_position(
        &self,
        id: &PatternId,
        pos: IndexPosition,
    ) -> Result<&Child, NoMatch> {
        self.children
            .get(id)
            .and_then(|p| p.get(pos))
            .ok_or(NoMatch::NoChildPatterns)
    }
    pub fn get_child_pattern(
        &self,
        id: &PatternId,
    ) -> Option<&Pattern> {
        self.children.get(id)
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
    ) -> Result<&mut Pattern, NoMatch> {
        self.children.get_mut(id).ok_or(NoMatch::NoChildPatterns)
    }
    pub fn expect_any_pattern(&self) -> (&PatternId, &Pattern) {
        self.children
            .iter()
            .next()
            .unwrap_or_else(|| panic!("Pattern vertex has no children {:#?}", self,))
    }
    pub fn expect_child_pattern(
        &self,
        id: &PatternId,
    ) -> &Pattern {
        self.get_child_pattern(id).unwrap_or_else(|| {
            panic!(
                "Child pattern with id {} does not exist in in vertex {:#?}",
                id, self,
            )
        })
    }
    pub fn expect_child_pattern_mut(
        &mut self,
        id: &PatternId,
    ) -> &mut Pattern {
        self.get_child_pattern_mut(id)
            .unwrap_or_else(|_| panic!("Child pattern with id {} does not exist in in vertex", id,))
    }
    pub fn get_children(&self) -> &ChildPatterns {
        &self.children
    }
    pub fn get_child_patterns<'a>(&'a self) -> impl IntoIterator<Item=Pattern> + 'a {
        clone_child_patterns(&self.children)
    }
    pub fn get_child_pattern_set(&self) -> HashSet<Pattern> {
        self.get_child_patterns().into_iter().collect()
    }
    pub fn get_child_pattern_vec(&self) -> Vec<Pattern> {
        self.get_child_patterns().into_iter().collect()
    }
    pub fn add_pattern<P: IntoPattern<Item = impl AsChild>>(
        &mut self,
        pat: P,
    ) -> PatternId {
        // TODO: detect unmatching pattern
        let id = Self::next_child_pattern_id();
        self.children.insert(id, pat.into_pattern());
        id
    }
    pub fn add_parent(
        &mut self,
        parent: impl AsChild,
        pattern: usize,
        index: PatternId,
    ) {
        if let Some(parent) = self.parents.get_mut(&parent.index()) {
            parent.add_pattern_index(pattern, index);
        } else {
            let mut parent_rel = Parent::new(parent.width());
            parent_rel.add_pattern_index(pattern, index);
            self.parents.insert(parent.index(), parent_rel);
        }
    }
    pub fn remove_parent(
        &mut self,
        vertex: impl Indexed,
        pattern: usize,
        index: PatternId,
    ) {
        if let Some(parent) = self.parents.get_mut(&vertex.index()) {
            if parent.pattern_indices.len() > 1 {
                parent.remove_pattern_index(pattern, index);
            } else {
                self.parents.remove(&vertex.index());
            }
        }
    }
    pub fn get_parents_below_width(
        &self,
        width_ceiling: Option<TokenPosition>,
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
    pub fn to_pattern_strings<T: Tokenize + std::fmt::Display>(
        &self,
        g: &Hypergraph<T>,
    ) -> Vec<Vec<String>> {
        self.get_children()
            .values()
            .map(|pat| {
                pat.iter()
                    .map(|c| g.index_string(c.index))
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>()
    }
    pub fn filter_parent_to(
        &self,
        parent_index: impl Indexed,
        cond: impl Fn(&&Parent) -> bool,
    ) -> Result<&'_ Parent, NoMatch> {
        let index = parent_index.index();
        self.get_parent(index)
            .ok()
            .filter(cond)
            .ok_or(NoMatch::NoMatchingParent(index))
    }
    pub fn get_parent_to_starting_at(
        &self,
        parent_index: impl Indexed,
        offset: PatternId,
    ) -> Result<&'_ Parent, NoMatch> {
        self.filter_parent_to(parent_index, |parent| parent.exists_at_pos(offset))
    }
    pub fn get_parent_to_ending_at(
        &self,
        parent_index: impl Indexed,
        offset: PatternId,
    ) -> Result<&'_ Parent, NoMatch> {
        self.filter_parent_to(parent_index, |parent| {
            offset
                .checked_sub(self.width)
                .map(|p| parent.exists_at_pos(p))
                .unwrap_or(false)
        })
    }
    pub fn get_parent_at_prefix_of(
        &self,
        index: impl Indexed,
    ) -> Result<&'_ Parent, NoMatch> {
        self.get_parent_to_starting_at(index, 0)
    }
    pub fn get_parent_at_postfix_of(
        &self,
        index: impl Indexed,
    ) -> Result<&'_ Parent, NoMatch> {
        self.filter_parent_to(index, |parent| {
            parent
                .width
                .checked_sub(self.width)
                .map(|p| parent.exists_at_pos(p))
                .unwrap_or(false)
        })
    }
    pub fn find_ancestor_with_range(
        &self,
        half: Pattern,
        range: impl PatternRangeIndex,
    ) -> Result<PatternId, NoMatch> {
        self.children
            .iter()
            .find_map(|(id, pat)| {
                if pat[range.clone()] == half[..] {
                    Some(*id)
                } else {
                    None
                }
            })
            .ok_or(NoMatch::NoChildPatterns)
    }
    pub fn largest_postfix(
        &self,
    ) -> (PatternId, Child) {
        let (id, c) = self.children
            .iter()
            .fold(None, |acc: Option<(&PatternId, &Child)>, (pid, p)| 
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
            )
            .unwrap();
        (*id, c.clone())
    }
}

impl<'g> Vertexed<'g, 'g> for &'g VertexData {
    fn vertex<T: Tokenize>(
        self,
        _graph: &'g Hypergraph<T>,
    ) -> &'g VertexData {
        self
    }
    fn vertex_ref<T: Tokenize>(
        &'g self,
        _graph: &'g Hypergraph<T>,
    ) -> &'g VertexData {
        *self
    }
}
impl<'g> Vertexed<'g, 'g> for &'g mut VertexData {
    fn vertex<T: Tokenize>(
        self,
        _graph: &'g Hypergraph<T>,
    ) -> &'g VertexData {
        self
    }
    fn vertex_ref<T: Tokenize>(
        &'g self,
        _graph: &'g Hypergraph<T>,
    ) -> &'g VertexData {
        *self
    }
}
impl<'g> VertexedMut<'g, 'g> for &'g mut VertexData {
    fn vertex_mut<T: Tokenize>(
        self,
        _graph: &'g mut Hypergraph<T>,
    ) -> &'g mut VertexData {
        self
    }
    fn vertex_ref_mut<T: Tokenize>(
        &'g mut self,
        _graph: &'g mut Hypergraph<T>,
    ) -> &'g mut VertexData {
        *self
    }
}
impl Indexed for VertexData {
    fn index(&self) -> VertexIndex {
        self.index
    }
}