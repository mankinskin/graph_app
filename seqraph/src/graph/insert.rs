use itertools::Itertools;

use crate::*;
use std::{
    collections::HashSet,
    sync::atomic::{
        AtomicUsize,
        Ordering,
    },
};

impl<'t, 'g, T> Hypergraph<T>
where
    T: Tokenize + 't,
{
    fn next_pattern_vertex_id() -> VertexIndex {
        static VERTEX_ID_COUNTER: AtomicUsize = AtomicUsize::new(0);
        VERTEX_ID_COUNTER.fetch_add(1, Ordering::Relaxed)
    }
    /// insert single token node
    pub fn insert_vertex(
        &mut self,
        key: VertexKey<T>,
        mut data: VertexData,
    ) -> Child {
        // TODO: return error if exists (don't overwrite by default)
        //let index = insert_full(key, data).0;
        let entry = self.graph.entry(key);
        data.index = entry.index();
        let c = Child::new(data.index, data.width);
        entry.or_insert(data);
        c
    }
    /// insert single token node
    pub fn insert_token(
        &mut self,
        token: Token<T>,
    ) -> Child {
        self.insert_vertex(VertexKey::Token(token), VertexData::new(0, 1))
    }
    /// insert multiple token nodes
    pub fn insert_tokens(
        &mut self,
        tokens: impl IntoIterator<Item = Token<T>>,
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
    /// add pattern to existing node
    pub fn add_pattern_to_node(
        &mut self,
        index: impl Indexed,
        indices: impl IntoIterator<Item = impl Indexed>,
    ) -> PatternId {
        // todo handle token nodes
        let (width, indices, children) = self.to_width_indices_children(indices);
        let data = self.expect_vertex_data_mut(index.index());
        let pattern_id = data.add_pattern(&children);
        self.add_parents_to_pattern_nodes(indices, Child::new(index, width), pattern_id);
        pattern_id
    }
    /// add pattern to existing node
    pub fn add_patterns_to_node(
        &mut self,
        index: impl Indexed,
        patterns: impl IntoIterator<Item = impl IntoIterator<Item = impl Indexed>>,
    ) -> Vec<PatternId> {
        let index = index.index();
        patterns
            .into_iter()
            .map(|p| self.add_pattern_to_node(index, p))
            .collect()
    }
    /// create new node from a pattern
    pub fn insert_pattern_with_id(
        &mut self,
        indices: impl IntoIterator<Item = impl Indexed>,
    ) -> (Child, Option<PatternId>) {
        let indices: Vec<_> = indices.into_iter().collect();
        if indices.len() == 1 {
            (self.to_child(indices.first().unwrap().index()), None)
        } else {
            let (c, id) = self.force_insert_pattern_with_id(indices);
            (c, Some(id))
        }
    }
    /// create new node from a pattern (even if single index)
    pub fn force_insert_pattern_with_id(
        &mut self,
        indices: impl IntoIterator<Item = impl Indexed>,
    ) -> (Child, PatternId) {
        let (width, indices, children) = self.to_width_indices_children(indices);
        let mut new_data = VertexData::new(0, width);
        let pattern_index = new_data.add_pattern(&children);
        let id = Self::next_pattern_vertex_id();
        let index = self.insert_vertex(VertexKey::Pattern(id), new_data);
        self.add_parents_to_pattern_nodes(indices, Child::new(index, width), pattern_index);
        (index, pattern_index)
    }
    /// create new node from a pattern
    pub fn insert_pattern(
        &mut self,
        indices: impl IntoIterator<Item = impl Indexed>,
    ) -> Child {
        self.insert_pattern_with_id(indices).0
    }
    /// create new node from a pattern
    pub fn force_insert_pattern(
        &mut self,
        indices: impl IntoIterator<Item = impl Indexed>,
    ) -> Child {
        self.force_insert_pattern_with_id(indices).0
    }
    pub fn insert_patterns_with_ids(
        &mut self,
        patterns: impl IntoIterator<Item = impl IntoIterator<Item = impl Indexed>>,
    ) -> (Child, Vec<PatternId>) {
        // todo handle token nodes
        let patterns = patterns.into_iter().collect_vec();
        let mut ids = Vec::with_capacity(patterns.len());
        let mut patterns = patterns.into_iter();
        let first = patterns.next().expect("Tried to insert no patterns");
        let (node, first_id) = self.insert_pattern_with_id(first);
        ids.push(first_id.unwrap());
        for pat in patterns {
            ids.push(self.add_pattern_to_node(&node, pat));
        }
        (node, ids)
    }
    /// create new node from multiple patterns
    pub fn insert_patterns(
        &mut self,
        patterns: impl IntoIterator<Item = impl IntoIterator<Item = impl Indexed>>,
    ) -> Child {
        // todo handle token nodes
        let mut patterns = patterns.into_iter();
        let first = patterns.next().expect("Tried to insert no patterns");
        let node = self.insert_pattern(first);
        for pat in patterns {
            self.add_pattern_to_node(&node, pat);
        }
        node
    }
    pub(crate) fn index_range_in(
        &mut self,
        parent: impl Indexed,
        pid: PatternId,
        range: impl PatternRangeIndex,
    ) -> Option<Child> {
        let vertex = self.expect_vertex_data(parent.index());
        let pattern = vertex.get_child_pattern_range(&pid, range.clone()).unwrap().to_vec();
        if pattern.is_empty() {
            None
        } else {
            let c = self.insert_pattern(pattern);
            self.replace_in_pattern(parent.index(), pid, range, c);
            Some(c)
        }
    }
    //pub(crate) fn replace_range_at(
    //    &mut self,
    //    loc: PatternLocation,
    //    range: impl PatternRangeIndex,
    //    rep: impl IntoPattern<Item = impl AsChild> + Clone,
    //) {
    //    self.replace_in_pattern(loc.parent, loc.pattern_id, range, rep)
    //}
    pub fn replace_in_pattern(
        &'g mut self,
        parent: impl Indexed,
        pat: PatternId,
        range: impl PatternRangeIndex,
        rep: impl IntoPattern<Item = impl AsChild> + Clone,
    ) {
        if range.start_bound() == range.end_bound() {
            // empty range
            return;
        }
        let parent_index = parent.index();
        let replace: Pattern = rep.into_pattern();
        let (old, width, start, rem) = {
            let vertex = self.expect_vertex_data_mut(parent);
            let width = vertex.width;
            let pattern = vertex.expect_child_pattern_mut(&pat);
            let old = pattern
                .get(range.clone())
                .expect("Replace range out of range of pattern!")
                .to_vec();
            *pattern = replace_in_pattern(pattern.as_pattern_view(), range.clone(), replace.clone());
            let start = range.clone().next().unwrap();
            (
                old,
                width,
                start,
                pattern.iter().skip(start + replace.len()).cloned().collect::<Pattern>(),
            )
        };
        let old_end = start + old.len();
        range.clone().zip(old.into_iter()).for_each(|(pos, c)| {
            let c = self.expect_vertex_data_mut(c);
            c.remove_parent(parent_index, pat, pos);
        });
        let new_end = start + replace.len();
        for (_i, c) in rem.into_iter().enumerate() {
            let indices = &mut self.expect_parent_mut(c, parent_index).pattern_indices;
            let drained: Vec<_> = indices.drain_filter(|i| i.pattern_id == pat && i.sub_index >= old_end)
                .map(|i| PatternIndex::new(i.pattern_id, new_end + i.sub_index - old_end))
                .collect();
            indices.extend(drained);
        }
        self.add_pattern_parent(Child::new(parent_index, width), replace, pat, start);
    }
    pub(crate) fn add_pattern_parent(
        &mut self,
        parent: impl AsChild,
        pattern: impl IntoPattern<Item = impl AsChild>,
        pattern_id: PatternId,
        start: usize,
    ) {
        pattern.into_iter().enumerate().for_each(|(pos, c)| {
            let pos = start + pos;
            let c = self.expect_vertex_data_mut(c);
            c.add_parent(parent.as_child(), pattern_id, pos);
        });
    }
    pub(crate) fn append_to_pattern(
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
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn insert_subpattern() {
        let mut graph = Hypergraph::default();
        if let [a, b, c, d] = graph.insert_tokens([
            Token::Element('a'),
            Token::Element('b'),
            Token::Element('c'),
            Token::Element('d'),
        ])[..]
        {
            let _abcd = graph.insert_pattern([a, b, c, d]);
            // read abcd
            // then abe
            // then bce
            // then cde
        } else {
            panic!()
        }
    }
}
