use serde::{
    de::DeserializeOwned,
    Deserialize,
    Serialize,
};
use std::fmt::{
    Debug,
    Display,
};
use crate::{
    direction::Right,
    graph::direction::index::IndexDirection,
};
use crate::graph::vertex::key::VertexKey;
use crate::graph::vertex::token::Tokenize;

pub trait GraphKind: Debug + Clone + Default + PartialEq + Eq {
    type Token: Tokenize + Display + DeserializeOwned;
    type Direction: IndexDirection;
}

pub type TokenOf<K> = <K as GraphKind>::Token;
pub type DefaultToken = TokenOf<BaseGraphKind>;
pub type DirectionOf<K> = <K as GraphKind>::Direction;
pub type DefaultDirection = DirectionOf<BaseGraphKind>;
pub type VertexKeyOf<G> = VertexKey<TokenOf<G>>;

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct BaseGraphKind;

impl GraphKind for BaseGraphKind {
    type Token = char;
    type Direction = Right;
}
