pub mod advanced;
pub mod child;
pub mod parent;
pub mod pattern;
pub mod token;
pub mod vertex;

use std::borrow::Borrow;

use derive_new::new;

use crate::{
    graph::{
        Hypergraph,
        getters::vertex::VertexSet,
        kind::GraphKind,
        vertex::{
            VertexIndex,
            child::Child,
            has_vertex_index::HasVertexIndex,
            pattern::{
                Pattern,
                id::PatternId,
            },
        },
    },
    path::structs::rooted::pattern_range::PatternRangePath,
};

#[derive(Debug, Clone, Eq, PartialEq, new)]
pub struct IndexWithPath {
    pub index: Child,
    pub path: PatternRangePath,
}
impl From<IndexWithPath> for Child {
    fn from(val: IndexWithPath) -> Self {
        val.index
    }
}
impl Borrow<Child> for IndexWithPath {
    fn borrow(&self) -> &Child {
        &self.index
    }
}
impl From<PatternRangePath> for IndexWithPath {
    fn from(path: PatternRangePath) -> Self {
        let index = *path.root.first().unwrap();
        IndexWithPath { index, path }
    }
}
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ErrorReason {
    EmptyPatterns,
    NoParents,
    NoChildPatterns,
    NotFound,
    ErrorReasoningParent(VertexIndex),
    InvalidPattern(PatternId),
    InvalidChild(usize),
    InvalidPatternRange(PatternId, Pattern, String),
    SingleIndex(Box<IndexWithPath>),
    ParentMatchingPartially,
    UnknownKey,
    UnknownIndex,
    UnknownToken,
    Unnecessary,
    EmptyRange,
}

impl<G: GraphKind> Hypergraph<G> {
    pub fn expect_index_width(
        &self,
        index: &impl HasVertexIndex,
    ) -> usize {
        self.expect_vertex(index.vertex_index()).width
    }
}

impl<G: GraphKind> Hypergraph<G> {
    //pub fn async_to_token_indices_stream(
    //    arc: Arc<RwLock<Self>>,
    //    tokens: impl TokenStream<T> + 't,
    //) -> impl PatternStream<VertexIndex, Token<T>> + 't {
    //    let handle = tokio::runtime::Handle::current();
    //    tokens.map(move |token|
    //        // is this slow?
    //        handle.block_on(async {
    //            arc.read().get_token_index(token.as_token())
    //                .map_err(|_| Token::Element(token))
    //        }))
    //}
    //pub fn async_to_token_children_stream(
    //    arc: Arc<RwLock<Self>>,
    //    tokens: impl TokenStream<T> + 't,
    //) -> impl PatternStream<Child, Token<T>> + 't {
    //    Self::async_to_token_indices_stream(arc, tokens)
    //
    //        .map(move |index| index.into_inner().map(|index| Child::new(index, 1)))
    //}
    //pub fn to_token_indices_stream(
    //    &'a self,
    //    tokens: impl TokenStream<G::Token> + 'a,
    //) -> impl PatternStream<VertexIndex, Token<G::Token>> + 'a {
    //    tokens.map(move |token| {
    //        self.get_token_index(token.as_token())
    //            .map_err(|_| Token::Element(token))
    //    })
    //}
    //pub fn to_token_children_stream(
    //    &'a self,
    //    tokens: impl TokenStream<G::Token> + 'a,
    //) -> impl PatternStream<Child, Token<G::Token>> + 'a {
    //    self.to_token_indices_stream(tokens)
    //        .map(move |index| index.into_inner().map(|index| Child::new(index, 1)))
    //}
}
