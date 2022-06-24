use crate::*;
use itertools::Itertools;
use petgraph::graph::DiGraph;
use std::{
    collections::HashMap,
    fmt::Debug, sync::{Arc, RwLock},
};

mod child_strings;
mod getters;
mod insert;
#[cfg(test)]
#[macro_use]
pub(crate) mod tests;

pub use {
    child_strings::*,
    getters::*,
    insert::*,
};

#[derive(Debug, Clone, Default)]
pub struct HypergraphRef<T: Tokenize>(pub Arc<RwLock<Hypergraph<T>>>);

impl<T: Tokenize> HypergraphRef<T> {
    pub fn new(g: Hypergraph<T>) -> Self {
        Self::from(g)
    }
}
impl<T: Tokenize> From<Hypergraph<T>> for HypergraphRef<T> {
    fn from(g: Hypergraph<T>) -> Self {
        Self(Arc::new(RwLock::new(g)))
    }
}
impl<T: Tokenize> std::ops::Deref for HypergraphRef<T> {
    type Target = Arc<RwLock<Hypergraph<T>>>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<T: Tokenize> std::ops::DerefMut for HypergraphRef<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
impl<T: Tokenize> std::convert::AsRef<Self> for Hypergraph<T> {
    fn as_ref(&self) -> &Self {
        self
    }
}
impl<T: Tokenize> std::convert::AsMut<Self> for Hypergraph<T> {
    fn as_mut(&mut self) -> &mut Self {
        self
    }
}

#[derive(Debug)]
pub struct Hypergraph<T: Tokenize> {
    graph: indexmap::IndexMap<VertexKey<T>, VertexData>,
}
use lazy_static::lazy_static;
lazy_static!{
    static ref LOGGER: Arc<RwLock<Logger>> = Arc::default();
}
impl<T: Tokenize> Default for Hypergraph<T> {
    fn default() -> Self {
        lazy_static::initialize(&LOGGER);
        Self {
            graph: indexmap::IndexMap::default(),
        }
    }
}
impl<'t, 'a, T: Tokenize> Hypergraph<T> {
    pub fn index_width(
        &self,
        index: &impl Indexed,
    ) -> TokenPosition {
        self.expect_vertex_data(index.index()).width
    }
    pub fn vertex_count(&self) -> usize {
        self.graph.len()
    }
    /// create index from a pattern and return its pattern id
    #[track_caller]
    pub fn index_pattern_with_id(
        &mut self,
        indices: impl IntoPattern,
    ) -> (Child, Option<PatternId>) {
        let (c, id) = self.insert_pattern_with_id(indices);
        (c.expect("Tried to index empty pattern!"), id)
    }
    /// create index from a pattern
    pub fn index_pattern(
        &mut self,
        indices: impl IntoPattern,
    ) -> Child {
        self.index_pattern_with_id(indices).0
    }
    //pub fn index_sequence<N: Into<T>, I: IntoIterator<Item = N>>(&mut self, seq: I) -> VertexIndex {
    //    let seq = seq.into_iter();
    //    let tokens = T::tokenize(seq);
    //    let pattern = self.to_token_children(tokens);
    //    self.index_pattern(&pattern[..])
    //}
}
impl<'t, 'a, T> Hypergraph<T>
where
    T: Tokenize + 't + std::fmt::Display,
{
    pub fn to_petgraph(&self) -> DiGraph<(VertexKey<T>, VertexData), Parent> {
        let mut pg = DiGraph::new();
        // id refers to index in Hypergraph
        // idx refers to index in petgraph
        let nodes: HashMap<_, _> = self
            .vertex_iter()
            .map(|(key, node)| {
                let idx = pg.add_node((*key, node.clone()));
                let id = self.expect_index_by_key(key);
                (id, (idx, node))
            })
            .collect();
        nodes.values().for_each(|(idx, node)| {
            let parents = node.get_parents();
            for (p_id, rel) in parents {
                let (p_idx, _p_data) = nodes
                    .get(p_id)
                    .expect("Parent not mapped to node in petgraph!");
                pg.add_edge(*p_idx, *idx, rel.clone());
            }
        });
        pg
    }

    pub fn to_node_child_strings(&self) -> ChildStrings {
        let nodes = self.graph.iter().map(|(key, data)| {
            (
                self.key_data_string(key, data),
                data.to_pattern_strings(self),
            )
        });
        ChildStrings::from_nodes(nodes)
    }
    pub fn pattern_child_strings(
        &self,
        pattern: impl IntoPattern,
    ) -> ChildStrings {
        let nodes = pattern.into_iter().map(|child| {
            (
                self.index_string(child.index()),
                self.expect_vertex_data(child.index())
                    .to_pattern_strings(self),
            )
        });
        ChildStrings::from_nodes(nodes)
    }

    pub(crate) fn pattern_string_with_separator(
        &'a self,
        pattern: impl IntoIterator<Item = impl Indexed>,
        separator: &'static str,
    ) -> String {
        pattern
            .into_iter()
            .map(|child| self.index_string(child.index()))
            .join(separator)
    }
    pub fn separated_pattern_string(
        &'a self,
        pattern: impl IntoIterator<Item = impl Indexed>,
    ) -> String {
        self.pattern_string_with_separator(pattern, "_")
    }
    pub fn pattern_string(
        &'a self,
        pattern: impl IntoIterator<Item = impl Indexed>,
    ) -> String {
        self.pattern_string_with_separator(pattern, "")
    }
    pub fn key_data_string(
        &self,
        key: &VertexKey<T>,
        data: &VertexData,
    ) -> String {
        self.key_data_string_impl(key, data, |t| t.to_string())
    }
    pub fn key_data_string_impl(
        &self,
        key: &VertexKey<T>,
        data: &VertexData,
        f: impl Fn(&Token<T>) -> String,
    ) -> String {
        match key {
            VertexKey::Token(token) => f(token),
            VertexKey::Pattern(_) => self.pattern_string(data.expect_any_pattern().1),
        }
    }
    pub fn index_string(
        &self,
        index: impl Indexed,
    ) -> String {
        let (key, data) = self.expect_vertex(index);
        self.key_data_string(key, data)
    }
    pub fn key_string(
        &self,
        key: &VertexKey<T>,
    ) -> String {
        let data = self.expect_vertex_data_by_key(key);
        self.key_data_string(key, data)
    }
}