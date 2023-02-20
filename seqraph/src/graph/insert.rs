use itertools::Itertools;

use crate::*;
use std::{
    sync::atomic::{
        AtomicUsize,
        Ordering,
    },
};
lazy_static! {
    static ref VERTEX_ID_COUNTER: Arc<Mutex<usize>> = Arc::new(Mutex::new(0));
}
impl<'t, 'g, G> Hypergraph<G>
where
    G: GraphKind,
{
    //fn next_pattern_vertex_id() -> VertexIndex {
    //    let mut lock = VERTEX_ID_COUNTER.lock().unwrap();
    //    let tmp = *lock;
    //    *lock += 1;
    //    tmp
    //}
    /// insert single token node
    pub fn insert_vertex(
        &mut self,
        key: VertexKey<G::Token>,
        mut data: VertexData,
    ) -> Child {
        assert!(!self.graph.contains_key(&key));
        let entry = self.graph.entry(key);
        data.index = entry.index();
        let c = Child::new(data.index, data.width);
        entry.or_insert(data);
        trace!(event = ?logger::Event::NewIndex);
        c
    }
    /// insert single token node
    pub fn insert_token(
        &mut self,
        token: Token<G::Token>,
    ) -> Child {
        self.insert_vertex(VertexKey::Token(token), VertexData::new(0, 1))
    }
    /// insert multiple token nodes
    pub fn insert_tokens(
        &mut self,
        tokens: impl IntoIterator<Item = Token<G::Token>>,
    ) -> Vec<Child> {
        tokens
            .into_iter()
            .map(|token| self.insert_vertex(VertexKey::Token(token), VertexData::new(0, 1)))
            .collect()
    }
    /// utility, builds total width, indices and children for pattern
    fn to_width_indices_children(
        &self,
        indices: impl IntoIterator<Item = impl Indexed>,
    ) -> (TokenPosition, Vec<VertexIndex>, Vec<Child>) {
        let mut width = 0;
        let (a, b) = indices
            .into_iter()
            .map(|index| {
                let index = index.index();
                let w = self.expect_vertex_data(index).get_width();
                width += w;
                (index, Child::new(index, w))
            })
            .unzip();
        (width, a, b)
    }
    /// adds a parent to all nodes in a pattern
    #[track_caller]
    fn add_parents_to_pattern_nodes(
        &mut self,
        pattern: Vec<VertexIndex>,
        parent: impl AsChild,
        pattern_id: PatternId,
    ) {
        for (i, child_index) in pattern.into_iter().enumerate() {
            let node = self.expect_vertex_data_mut(child_index);
            node.add_parent(parent.as_child(), pattern_id, i);
        }
    }
    pub fn validate_vertex(
        &self,
        index: impl Indexed,
    ) {
        self.expect_vertex_data(index).validate()
    }
    /// add pattern to existing node
    pub fn add_pattern_with_update(
        &mut self,
        index: impl Indexed,
        indices: impl IntoPattern,
    ) -> PatternId {
        // todo handle token nodes
        let (width, indices, children) = self.to_width_indices_children(indices);
        let data = self.expect_vertex_data_mut(index.index());
        let pattern_id = data.add_pattern_no_update(children);
        self.add_parents_to_pattern_nodes(indices, Child::new(index, width), pattern_id);
        pattern_id
    }
    /// add pattern to existing node
    #[track_caller]
    pub fn add_patterns_with_update(
        &mut self,
        index: impl Indexed,
        patterns: impl IntoIterator<Item = impl IntoPattern>,
    ) -> Vec<PatternId> {
        let index = index.index();
        patterns
            .into_iter()
            .map(|p| self.add_pattern_with_update(index, p))
            .collect()
    }
    /// create new node from a pattern
    #[track_caller]
    pub fn insert_pattern_with_id(
        &mut self,
        indices: impl IntoPattern,
    ) -> (Child, Option<PatternId>) {
        let indices = indices.into_pattern();
        let (c, id) = match indices.len() {
            0 => (None, None),
            1 => (Some(self.to_child(indices.first().unwrap().index())), None),
            _ => {
                let (c, id) = self.force_insert_pattern_with_id(indices);
                (Some(c), Some(id))
            }
        };
        (c.expect("Tried to index empty pattern!"), id)
    }
    /// create new node from a pattern (even if single index)
    #[track_caller]
    pub fn force_insert_pattern_with_id(
        &mut self,
        indices: impl IntoPattern,
    ) -> (Child, PatternId) {
        let (width, indices, children) = self.to_width_indices_children(indices);
        let mut new_data = VertexData::new(0, width);
        let pattern_index = new_data.add_pattern_no_update(children);
        let id = self.vertex_count(); //Self::next_pattern_vertex_id();
        let index = self.insert_vertex(VertexKey::Pattern(id), new_data);
        self.add_parents_to_pattern_nodes(indices, Child::new(index, width), pattern_index);
        (index, pattern_index)
    }
    /// create new node from a pattern
    pub fn insert_pattern(
        &mut self,
        indices: impl IntoPattern,
    ) -> Child {
        self.insert_pattern_with_id(indices).0
    }
    /// create new node from a pattern
    pub fn force_insert_pattern(
        &mut self,
        indices: impl IntoPattern,
    ) -> Child {
        self.force_insert_pattern_with_id(indices).0
    }
    pub fn insert_patterns_with_ids(
        &mut self,
        patterns: impl IntoIterator<Item = impl IntoPattern>,
    ) -> (Child, Vec<PatternId>) {
        // todo handle token nodes
        let patterns = patterns.into_iter().collect_vec();
        let mut ids = Vec::with_capacity(patterns.len());
        let mut patterns = patterns.into_iter();
        let first = patterns.next().expect("Tried to insert no patterns");
        let (node, first_id) = self.insert_pattern_with_id(first);
        ids.push(first_id.unwrap());
        for pat in patterns {
            ids.push(self.add_pattern_with_update(&node, pat));
        }
        (node, ids)
    }
    /// create new node from multiple patterns
    #[track_caller]
    pub fn insert_patterns(
        &mut self,
        patterns: impl IntoIterator<Item = impl IntoPattern>,
    ) -> Child {
        let patterns = patterns.into_iter()
            .map(IntoPattern::into_pattern)
            .collect_vec();
        patterns.iter().find(|p| p.len() == 1)
            .map(|p| p.first().unwrap().clone())
            .unwrap_or_else(|| {
                // todo handle token nodes
                let mut patterns = patterns.into_iter();
                let first = patterns.next().expect("Tried to insert no patterns");
                let node = self.insert_pattern(first);
                for pat in patterns {
                    self.add_pattern_with_update(&node, pat);
                }
                node
            })

    }
    #[track_caller]
    pub fn try_insert_patterns(
        &mut self,
        patterns: impl IntoIterator<Item = impl IntoPattern>,
    ) -> Option<Child> {
        let patterns = patterns.into_iter()
            .map(IntoPattern::into_pattern)
            .collect_vec();
        if patterns.is_empty() {
            None
        } else {
            Some(self.insert_patterns(patterns))
        }
    }
    #[track_caller]
    pub fn try_insert_range_in(
        &mut self,
        location: impl IntoPatternLocation,
        range: impl PatternRangeIndex,
    ) -> Result<Result<Child, Child>, NoMatch> {
        let location = location.into_pattern_location();
        let vertex = self.expect_vertex_data(location.parent);
        vertex.get_child_pattern(&location.pattern_id)
            .map(|pattern| pattern.to_vec())
            .and_then(|pattern|
                pattern::get_child_pattern_range(
                    &location.pattern_id,
                    pattern.borrow(),
                    range.clone()
                )
                .and_then(|inner|
                    if inner.is_empty() {
                        Err(NoMatch::EmptyRange)
                    } else if inner.len() == 1 {
                        Ok(Ok(*inner.first().unwrap()))
                    } else if pattern.len() > inner.len() {
                        let c = self.insert_pattern(inner);
                        self.replace_in_pattern(location, range, c);
                        Ok(Ok(c))
                    } else {
                        Ok(Err(location.parent))
                    }
                )
            )
    }
    #[track_caller]
    pub fn insert_range_in(
        &mut self,
        location: impl IntoPatternLocation,
        range: impl PatternRangeIndex,
    ) -> Result<Child, NoMatch> {
        self.try_insert_range_in(
            location,
            range,
        )
        .and_then(|c| c.or(Err(NoMatch::Unnecessary)))
    }
    #[track_caller]
    pub fn insert_range_in_or_default(
        &mut self,
        location: impl IntoPatternLocation,
        range: impl PatternRangeIndex,
    ) -> Result<Child, NoMatch> {
        self.try_insert_range_in(
            location,
            range,
        )
        .map(|c| match c {
            Ok(c) => c,
            Err(c) => c,
        })
    }
    //#[track_caller]
    pub fn replace_in_pattern(
        &'g mut self,
        location: impl IntoPatternLocation,
        range: impl PatternRangeIndex,
        replace: impl IntoPattern + Clone,
    ) {
        let location = location.into_pattern_location();
        let parent = location.parent;
        let parent_index = parent.index();
        let pat = location.pattern_id;
        let (replaced, width, start, new_end, rem) = {
            let vertex = self.expect_vertex_data_mut(parent);
            let width = vertex.width;
            let pattern = vertex.expect_child_pattern_mut(&pat);
            let _backup = pattern.clone();
            let start = range.clone().next().unwrap();
            let new_end = start + replace.borrow().len();
            let _old = pattern.clone();
            let replaced = replace_in_pattern(&mut *pattern, range.clone(), replace.clone());
            let rem = pattern.iter().skip(new_end).cloned().collect::<Pattern>();
            vertex.validate();
            (
                replaced,
                width,
                start,
                new_end,
                rem,
            )
        };
        let old_end = start + replaced.len();
        range.clone().zip(replaced.into_iter()).for_each(|(pos, c)| {
            let c = self.expect_vertex_data_mut(c);
            c.remove_parent_index(parent_index, pat, pos);
        });
        for c in rem.into_iter().unique() {
            let c = self.expect_vertex_data_mut(c);
            let indices = &mut c.expect_parent_mut(parent_index).pattern_indices;
            *indices = indices.drain()
                .filter(|i| i.pattern_id != pat || !range.clone().contains(&i.sub_index))
                .map(|i|
                    if i.pattern_id == pat && i.sub_index >= old_end {
                        PatternIndex::new(i.pattern_id, i.sub_index - old_end + new_end)
                    } else {
                        i
                    }
                )
                .collect();
            if indices.is_empty() {
                c.remove_parent(parent_index);
            }
        }
        self.add_pattern_parent(Child::new(parent_index, width), replace, pat, start);
        self.validate_expansion(parent_index);
    }
    pub fn add_pattern_parent(
        &mut self,
        parent: impl AsChild,
        pattern: impl IntoPattern,
        pattern_id: PatternId,
        start: usize,
    ) {
        pattern.into_iter().enumerate().for_each(|(pos, c)| {
            let pos = start + pos;
            let c = self.expect_vertex_data_mut(c);
            c.add_parent(parent.as_child(), pattern_id, pos);
        });
    }
    pub fn append_to_pattern(
        &mut self,
        parent: impl AsChild,
        pattern_id: PatternId,
        new: impl IntoIterator<Item = impl AsChild>,
    ) -> Child {
        let new: Vec<_> = new.into_iter().map(|c| c.to_child()).collect();
        if new.is_empty() {
            return parent.to_child();
        }
        let width = pattern_width(&new);
        let (offset, width) = {
            // Todo: use smart pointers to reference data in the graph
            // so we can mutate multiple different nodes at the same time
            let vertex = self.expect_vertex_data(parent.index());
            let pattern = vertex.expect_child_pattern(&pattern_id).clone();
            for c in pattern.into_iter().collect::<HashSet<_>>() {
                let c = self.expect_vertex_data_mut(c);
                c.get_parent_mut(parent.index()).unwrap().width += width;
            }
            let vertex = self.expect_vertex_data_mut(parent.index());
            let pattern = vertex.expect_child_pattern_mut(&pattern_id);
            let offset = pattern.len();
            pattern.extend(new.iter());
            vertex.width += width;
            (offset, vertex.width)
        };
        let parent = Child::new(parent.index(), width);
        self.add_pattern_parent(parent, new, pattern_id, offset);
        parent
    }
    pub fn new_token_indices(
        &mut self,
        sequence: impl IntoIterator<Item = G::Token>,
    ) -> NewTokenIndices {
        sequence
            .into_iter()
            .map(|t| Token::Element(t))
            .map(|t| match {
                self.get_token_index(t)
            } {
                Ok(i) => NewTokenIndex::Known(i),
                Err(_) => {
                    let i = self.insert_token(t);
                    NewTokenIndex::New(i.index)
                }
            })
            .collect()
    }
}