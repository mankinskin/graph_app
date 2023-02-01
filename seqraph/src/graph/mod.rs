use crate::*;
use itertools::Itertools;
use petgraph::graph::DiGraph;
use std::{
    fmt::Debug,
};

type HashMap<K, V> = DeterministicHashMap<K, V>;
mod child_strings;
mod getters;
mod insert;
mod validation;
pub mod kind;

#[cfg(test)]
#[macro_use]
pub mod tests;

pub use {
    child_strings::*,
    getters::*,
    insert::*,
    kind::*,
};

#[derive(Debug, Clone, Default)]
pub struct HypergraphRef<G: GraphKind = BaseGraphKind>(pub Arc<RwLock<Hypergraph<G>>>);

impl<G: GraphKind> HypergraphRef<G> {
    pub fn new(g: Hypergraph<G>) -> Self {
        Self::from(g)
    }
}
impl<G: GraphKind> From<Hypergraph<G>> for HypergraphRef<G> {
    fn from(g: Hypergraph<G>) -> Self {
        Self(Arc::new(RwLock::new(g)))
    }
}
impl<G: GraphKind> std::ops::Deref for HypergraphRef<G> {
    type Target = Arc<RwLock<Hypergraph<G>>>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<G: GraphKind> std::ops::DerefMut for HypergraphRef<G> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
impl<G: GraphKind> std::convert::AsRef<Self> for Hypergraph<G> {
    fn as_ref(&self) -> &Self {
        self
    }
}
impl<G: GraphKind> std::convert::AsMut<Self> for Hypergraph<G> {
    fn as_mut(&mut self) -> &mut Self {
        self
    }
}

#[derive(Debug)]
pub struct Hypergraph<G: GraphKind = BaseGraphKind> {
    graph: indexmap::IndexMap<VertexKey<G::Token>, VertexData>,
    _ty: std::marker::PhantomData<G>,
}
lazy_static! {
    static ref LOGGER: Arc<RwLock<Logger>> = Arc::default();
}
impl<G: GraphKind> Default for Hypergraph<G> {
    fn default() -> Self {
        lazy_static::initialize(&LOGGER);
        Self {
            graph: indexmap::IndexMap::default(),
            _ty: Default::default(),
        }
    }
}
impl<'t, 'a, G: GraphKind> Hypergraph<G> {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn index_width(
        &self,
        index: &impl Indexed,
    ) -> TokenPosition {
        self.expect_vertex_data(index.index()).width
    }
    pub fn vertex_count(&self) -> usize {
        self.graph.len()
    }
    //pub fn index_sequence<N: Into<G>, I: IntoIterator<Item = N>>(&mut self, seq: I) -> VertexIndex {
    //    let seq = seq.into_iter();
    //    let tokens = T::tokenize(seq);
    //    let pattern = self.to_token_children(tokens);
    //    self.index_pattern(&pattern[..])
    //}
    pub fn insert_token_indices(
        &self,
        index: impl AsChild,
    ) -> Vec<VertexIndex> {
        if index.width() == 1 {
            vec![index.index()]
        } else {
            let data = self.expect_vertex_data(index);
            assert!(!data.children.is_empty());
            data.children.values().fold(None, |acc, p| {
                let exp = self.pattern_token_indices(p.borrow());
                acc.map(|acc| {
                    assert_eq!(acc, exp);
                    acc
                }).or(Some(exp.clone()))
            }).unwrap()
        }
    }
    pub fn pattern_token_indices(
        &self,
        pattern: impl IntoPattern,
    ) -> Vec<VertexIndex> {
        pattern.into_iter().flat_map(|c|
            self.insert_token_indices(c)
        ).collect_vec()
    }
    pub fn validate_expansion(&self, index: impl Indexed) {
        //let root = index.index();
        let data = self.expect_vertex_data(index);
        data.children
            .iter()
            .fold(Vec::new(), |mut acc: Vec<VertexIndex>, (_pid, p)| {
                assert!(!p.is_empty());
                let exp = self.pattern_token_indices(p.borrow());
                if acc.is_empty() {
                    acc = exp;
                } else {
                    assert_eq!(acc, exp);
                }
                acc
            });
    }
}

#[derive(Clone, Debug)]
pub struct Edge {
    pub parent: Parent,
    pub child: Child,
}
impl<'t, 'a, G> Hypergraph<G>
where
    G: GraphKind,
    G::Token: std::fmt::Display + 't,
{
    pub fn to_petgraph(&self) -> DiGraph<(VertexKey<G::Token>, VertexData), Edge> {
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
            for (p_id, parent) in parents {
                let (p_idx, _p_data) = nodes
                    .get(p_id)
                    .expect("Parent not mapped to node in petgraph!");
                pg.add_edge(*p_idx, *idx, Edge {
                    parent: parent.clone(),
                    child: node.as_child(),
                });
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

    pub fn pattern_string_with_separator(
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
    pub fn pattern_strings(
        &'a self,
        patterns: impl IntoIterator<Item = impl IntoIterator<Item = impl Indexed>>,
    ) -> Vec<String> {
        patterns
            .into_iter()
            .map(|pattern|
                self.pattern_string_with_separator(pattern, "")
            )
            .collect()
    }
    pub fn key_data_string(
        &self,
        key: &VertexKey<G::Token>,
        data: &VertexData,
    ) -> String {
        self.key_data_string_impl(key, data, |t| t.to_string())
    }
    pub fn key_data_string_impl(
        &self,
        key: &VertexKey<G::Token>,
        data: &VertexData,
        f: impl Fn(&Token<G::Token>) -> String,
    ) -> String {
        match key {
            VertexKey::Token(token) => f(token),
            VertexKey::Pattern(_) => self.pattern_string(data.expect_any_child_pattern().1),
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
        key: &VertexKey<G::Token>,
    ) -> String {
        let data = self.expect_vertex_data_by_key(key);
        self.key_data_string(key, data)
    }

}