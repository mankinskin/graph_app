use std::{
    borrow::Borrow,
    sync::{
        Arc,
        atomic::{
            self,
            AtomicUsize,
        },
        RwLock,
    },
};

use itertools::Itertools;
use petgraph::graph::DiGraph;

use crate::{
    graph::{
        child_strings::ChildStrings,
        kind::{
            BaseGraphKind,
            GraphKind,
        },
    },
    HashMap,
    vertex::{
        child::Child,
        indexed::{
            AsChild,
            Indexed,
        },
        pattern::IntoPattern,
        PatternId,
        token::Token,
    },
};

pub mod child_strings;
pub mod direction;
pub mod getters;
pub mod insert;
pub mod kind;
pub mod validation;

#[cfg(test)]
#[macro_use]
pub mod tests;

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

impl<G: GraphKind> AsRef<Self> for Hypergraph<G> {
    fn as_ref(&self) -> &Self {
        self
    }
}

impl<G: GraphKind> AsMut<Self> for Hypergraph<G> {
    fn as_mut(&mut self) -> &mut Self {
        self
    }
}

#[derive(Debug)]
pub struct Hypergraph<G: GraphKind = BaseGraphKind> {
    graph: indexmap::IndexMap<crate::vertex::VertexIndex, crate::vertex::VertexData<G>>,
    tokens: indexmap::IndexMap<Token<G::Token>, crate::vertex::VertexIndex>,
    pattern_id_count: AtomicUsize,
    vertex_id_count: AtomicUsize,
    _ty: std::marker::PhantomData<G>,
}

impl<G: GraphKind> Default for Hypergraph<G> {
    fn default() -> Self {
        Self {
            graph: indexmap::IndexMap::default(),
            tokens: indexmap::IndexMap::default(),
            pattern_id_count: AtomicUsize::new(0),
            vertex_id_count: AtomicUsize::new(0),
            _ty: Default::default(),
        }
    }
}

impl<'t, 'a, G: GraphKind> Hypergraph<G> {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn to_ref(self) -> HypergraphRef<G> {
        self.into()
    }
    pub fn vertex_count(&self) -> usize {
        self.graph.len()
    }
    pub fn next_vertex_id(&mut self) -> crate::vertex::VertexIndex {
        self.vertex_id_count.fetch_add(1, atomic::Ordering::SeqCst)
    }
    pub fn next_pattern_id(&mut self) -> PatternId {
        self.pattern_id_count.fetch_add(1, atomic::Ordering::SeqCst)
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
    ) -> Vec<crate::vertex::VertexIndex> {
        if index.width() == 1 {
            vec![index.vertex_index()]
        } else {
            let data = self.expect_vertex_data(index);
            assert!(!data.children.is_empty());
            data.children
                .values()
                .fold(None, |acc, p| {
                    let exp = self.pattern_token_indices(p.borrow());
                    acc.map(|acc| {
                        assert_eq!(acc, exp);
                        acc
                    })
                        .or(Some(exp.clone()))
                })
                .unwrap()
        }
    }
    pub fn pattern_token_indices(
        &self,
        pattern: impl IntoPattern,
    ) -> Vec<crate::vertex::VertexIndex> {
        pattern
            .into_iter()
            .flat_map(|c| self.insert_token_indices(c))
            .collect_vec()
    }
    pub fn validate_expansion(
        &self,
        index: impl Indexed,
    ) {
        //let root = index.index();
        let data = self.expect_vertex_data(index);
        data.children.iter().fold(
            Vec::new(),
            |mut acc: Vec<crate::vertex::VertexIndex>, (_pid, p)| {
                assert!(!p.is_empty());
                let exp = self.pattern_token_indices(p.borrow());
                if acc.is_empty() {
                    acc = exp;
                } else {
                    assert_eq!(acc, exp);
                }
                acc
            },
        );
    }
}

#[derive(Clone, Debug)]
pub struct Edge {
    pub parent: crate::vertex::parent::Parent,
    pub child: Child,
}

impl<'t, 'a, G: GraphKind> Hypergraph<G>
    where
        G::Token: std::fmt::Display + 't,
{
    pub fn to_petgraph(
        &self
    ) -> DiGraph<(crate::vertex::VertexIndex, crate::vertex::VertexData<G>), Edge> {
        let mut pg = DiGraph::new();
        // id refers to index in Hypergraph
        // idx refers to index in petgraph
        let nodes: HashMap<_, (_, &crate::vertex::VertexData<G>)> = self
            .vertex_iter()
            .map(|(id, node)| {
                let idx = pg.add_node((*id, node.clone()));
                (id, (idx, node))
            })
            .collect();
        nodes.values().for_each(|(idx, node)| {
            let parents = node.get_parents();
            for (p_id, parent) in parents {
                let (p_idx, _p_data) = nodes
                    .get(p_id)
                    .expect("Parent not mapped to node in petgraph!");
                pg.add_edge(
                    *p_idx,
                    *idx,
                    Edge {
                        parent: parent.clone(),
                        child: node.as_child(),
                    },
                );
            }
        });
        pg
    }

    pub fn to_node_child_strings(&self) -> ChildStrings {
        let nodes = self
            .graph
            .iter()
            .map(|(_, data)| (self.vertex_data_string(data), data.to_pattern_strings(self)));
        ChildStrings::from_nodes(nodes)
    }
    pub fn pattern_child_strings(
        &self,
        pattern: impl IntoPattern,
    ) -> ChildStrings {
        let nodes = pattern.into_iter().map(|child| {
            (
                self.index_string(child.vertex_index()),
                self.expect_vertex_data(child.vertex_index())
                    .to_pattern_strings(self),
            )
        });
        ChildStrings::from_nodes(nodes)
    }

    pub fn pattern_string_with_separator(
        &'a self,
        pattern: impl IntoIterator<Item=impl Indexed>,
        separator: &'static str,
    ) -> String {
        pattern
            .into_iter()
            .map(|child| self.index_string(child.vertex_index()))
            .join(separator)
    }
    pub fn separated_pattern_string(
        &'a self,
        pattern: impl IntoIterator<Item=impl Indexed>,
    ) -> String {
        self.pattern_string_with_separator(pattern, "_")
    }
    pub fn pattern_string(
        &'a self,
        pattern: impl IntoIterator<Item=impl Indexed>,
    ) -> String {
        self.pattern_string_with_separator(pattern, "")
    }
    pub fn pattern_strings(
        &'a self,
        patterns: impl IntoIterator<Item=impl IntoIterator<Item=impl Indexed>>,
    ) -> Vec<String> {
        patterns
            .into_iter()
            .map(|pattern| self.pattern_string_with_separator(pattern, ""))
            .collect()
    }
    pub fn vertex_data_string(
        &self,
        data: &crate::vertex::VertexData<G>,
    ) -> String {
        if let Some(token) = data.token {
            token.to_string()
        } else {
            self.pattern_string(data.expect_any_child_pattern().1)
        }
    }
    pub fn index_string(
        &self,
        index: impl Indexed,
    ) -> String {
        let data = self.expect_vertex_data(index);
        self.vertex_data_string(data)
    }
}
