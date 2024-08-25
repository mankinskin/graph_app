use std::borrow::Borrow;
use derive_new::new;
use serde::{Deserialize, Serialize};
use crate::graph::kind::{BaseGraphKind, TokenOf};
use crate::graph::vertex::{
    child::Child,
    has_vertex_index::HasVertexIndex,
    token::{Token, Tokenize},
    VertexIndex,
};

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy, new, Serialize, Deserialize)]
pub enum VertexKey<T: Tokenize = TokenOf<BaseGraphKind>> {
    Pattern(Child),
    Token(Token<T>, VertexIndex)
}
impl<T: Tokenize> HasVertexIndex for VertexKey<T> {
    fn vertex_index(&self) -> VertexIndex {
        match self {
            Self::Token(_token, index) => *index,
            Self::Pattern(child) => child.vertex_index(),
        }
    }
}
impl<T: Tokenize> Borrow<VertexIndex> for VertexKey<T> {
    fn borrow(&self) -> &VertexIndex {
        match self {
            Self::Token(_token, index) => index,
            Self::Pattern(child) => &child.index,
        }
    }
}

