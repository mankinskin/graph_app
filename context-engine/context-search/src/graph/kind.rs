use crate::{
    direction::{
        pattern::PatternDirection,
        Right,
    },
    graph::vertex::token::Tokenize,
};
use serde::{
    de::DeserializeOwned,
    Deserialize,
    Serialize,
};
use std::fmt::{
    Debug,
    Display,
};

pub trait GraphKind: Debug + Clone + Default + PartialEq + Eq {
    type Token: Tokenize + Display + DeserializeOwned;
    type Direction: PatternDirection;
}

pub type TokenOf<K> = <K as GraphKind>::Token;
pub type DefaultToken = TokenOf<BaseGraphKind>;
pub type DirectionOf<K> = <K as GraphKind>::Direction;
pub type DefaultDirection = DirectionOf<BaseGraphKind>;

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct BaseGraphKind;

impl GraphKind for BaseGraphKind {
    type Token = char;
    type Direction = Right;
}
