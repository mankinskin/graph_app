use std::sync::{
    Arc,
    RwLock,
};

use itertools::Itertools;
use petgraph::graph::DiGraph;
use serde::{
    Deserialize,
    Serialize,
};
use vertex::{
    child::Child,
    has_vertex_index::{
        HasVertexIndex,
        ToChild,
    },
    pattern::IntoPattern,
    token::Token,
    VertexIndex,
};
use crate::{
    graph::{
        child_strings::ChildStrings,
        kind::{
            BaseGraphKind,
            GraphKind,
        },
    },
    HashMap,
};
use crate::graph::{
    vertex::data::VertexData,
    vertex::key::VertexKey,
    getters::vertex::VertexSet,
};

pub mod child_strings;
pub mod getters;
pub mod insert;
pub mod kind;
pub mod validation;


pub mod vertex;

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

#[derive(Debug, Serialize, Deserialize)]
pub struct Hypergraph<G: GraphKind = BaseGraphKind> {
    graph: indexmap::IndexMap<VertexKey, VertexData>,
    tokens: indexmap::IndexMap<VertexKey, Token<G::Token>>,
    token_keys: indexmap::IndexMap<Token<G::Token>, VertexKey>,
    _ty: std::marker::PhantomData<G>,
}
impl<G: GraphKind> Clone for Hypergraph<G> {
    fn clone(
        &self,
    ) -> Self {
        Self {
            graph: self.graph.clone(),
            tokens: self.tokens.clone(),
            token_keys: self.token_keys.clone(),
            //pattern_id_count: self.pattern_id_count.load(Ordering::SeqCst).clone().into(),
            //vertex_id_count: self.vertex_id_count.load(Ordering::SeqCst).clone().into(),
            _ty: self._ty,
        }
    }
}
impl<G: GraphKind> PartialEq for Hypergraph<G> {
    fn eq(
        &self,
        other: &Self,
    ) -> bool {
        self.graph.eq(&other.graph)
            && self.tokens.eq(&other.tokens)
            && self.token_keys.eq(&other.token_keys)
            //&& self
            //    .pattern_id_count
            //    .load(Ordering::SeqCst)
            //    .eq(&other.pattern_id_count.load(Ordering::SeqCst))
            //&& self
            //    .vertex_id_count
            //    .load(Ordering::SeqCst)
            //    .eq(&other.vertex_id_count.load(Ordering::SeqCst))
            && self._ty.eq(&other._ty)
    }
}
impl<G: GraphKind> Eq for Hypergraph<G> {}

impl<G: GraphKind> Default for Hypergraph<G> {
    fn default() -> Self {
        Self {
            graph: indexmap::IndexMap::default(),
            tokens: indexmap::IndexMap::default(),
            token_keys: indexmap::IndexMap::default(),
            //pattern_id_count: AtomicUsize::new(0),
            //vertex_id_count: AtomicUsize::new(0),
            _ty: Default::default(),
        }
    }
}

impl<G: GraphKind> Hypergraph<G> {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn to_ref(self) -> HypergraphRef<G> {
        self.into()
    }
    pub fn vertex_count(&self) -> usize {
        self.graph.len()
    }
    //pub fn next_vertex_id(&mut self) -> vertex::VertexIndex {
    //    self.vertex_id_count.fetch_add(1, atomic::Ordering::SeqCst)
    //}
    //pub fn next_pattern_id(&mut self) -> PatternId {
    //    self.pattern_id_count.fetch_add(1, atomic::Ordering::SeqCst)
    //}
    //pub fn index_sequence<N: Into<G>, I: IntoIterator<Item = N>>(&mut self, seq: I) -> VertexIndex {
    //    let seq = seq.into_iter();
    //    let tokens = T::tokenize(seq);
    //    let pattern = self.to_token_children(tokens);
    //    self.index_pattern(&pattern[..])
    //}
    //pub fn insert_token_indices(
    //    &self,
    //    index: impl ToChild,
    //) -> Vec<VertexIndex> {
    //    if index.width() == 1 {
    //        vec![index.vertex_index()]
    //    } else {
    //        let data = self.expect_vertex(index);
    //        assert!(!data.children.is_empty());
    //        data.children
    //            .values()
    //            .fold(None, |acc, p| {
    //                let exp = self.pattern_token_indices(p.borrow());
    //                acc.map(|acc| {
    //                    assert_eq!(acc, exp);
    //                    acc
    //                })
    //                .or(Some(exp.clone()))
    //            })
    //            .unwrap()
    //    }
    //}
    //pub fn pattern_token_indices(
    //    &self,
    //    pattern: impl IntoPattern,
    //) -> Vec<VertexIndex> {
    //    pattern
    //        .into_iter()
    //        .flat_map(|c| self.insert_token_indices(c))
    //        .collect_vec()
    //}
    pub fn validate_expansion(
        &self,
        index: impl HasVertexIndex,
    ) {
        //let root = index.index();
        let data = self.expect_vertex(index.vertex_index());
        data.children.iter().fold(
            Vec::new(),
            |mut acc: Vec<vertex::VertexIndex>, (_pid, p)| {
                assert!(!p.is_empty());
                let exp = p.iter().map(|c| c.vertex_index()).collect_vec();
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
    pub parent: vertex::parent::Parent,
    pub child: Child,
}

impl<'a, G: GraphKind> Hypergraph<G>
where
    G::Token: std::fmt::Display,
{
    pub fn to_petgraph(
        &self
    ) -> DiGraph<(VertexIndex, VertexData), Edge> {
        let mut pg = DiGraph::new();
        // id refers to index in Hypergraph
        // idx refers to index in petgraph
        let nodes: HashMap<_, (_, &VertexData)> = self
            .vertex_iter()
            .map(|(_id, node)| {
                let idx = pg.add_node((node.vertex_index(), node.clone()));
                (node.vertex_index(), (idx, node))
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
                        child: node.to_child(),
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
                self.expect_vertex(child.vertex_index())
                    .to_pattern_strings(self),
            )
        });
        ChildStrings::from_nodes(nodes)
    }

    pub fn pattern_string_with_separator(
        &'a self,
        pattern: impl IntoIterator<Item = impl HasVertexIndex>,
        separator: &'static str,
    ) -> String {
        pattern
            .into_iter()
            .map(|child| self.index_string(child.vertex_index()))
            .join(separator)
    }
    pub fn separated_pattern_string(
        &'a self,
        pattern: impl IntoIterator<Item = impl HasVertexIndex>,
    ) -> String {
        self.pattern_string_with_separator(pattern, "_")
    }
    pub fn pattern_string(
        &'a self,
        pattern: impl IntoIterator<Item = impl HasVertexIndex>,
    ) -> String {
        self.pattern_string_with_separator(pattern, "")
    }
    pub fn pattern_strings(
        &'a self,
        patterns: impl IntoIterator<Item = impl IntoIterator<Item = impl HasVertexIndex>>,
    ) -> Vec<String> {
        patterns
            .into_iter()
            .map(|pattern| self.pattern_string_with_separator(pattern, ""))
            .collect()
    }
    pub fn get_token_by_key(&self, key: &VertexKey) -> Option<&Token<G::Token>> {
        self.tokens.get(key)
    }
    pub fn expect_token_by_key(&self, key: &VertexKey) -> &Token<G::Token> {
        self.get_token_by_key(key)
            .expect("Key does not belong to a token!")
    }
    pub fn vertex_key_string(
        &self,
        key: &VertexKey,
    ) -> String {
        self.vertex_data_string(self.expect_vertex(key))
    }
    pub fn vertex_data_string(
        &self,
        data: &VertexData,
    ) -> String {
        if let Some(token) = self.get_token_by_key(&data.key) {
            token.to_string()
        } else {
            assert!(data.width > 1);
            self.pattern_string(data.expect_any_child_pattern().1)
        }
    }
    pub fn index_string(
        &self,
        index: impl HasVertexIndex,
    ) -> String {
        let data = self.expect_vertex(index.vertex_index());
        self.vertex_data_string(data)
    }
}
