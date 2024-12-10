use serde::{
    de::DeserializeOwned,
    Deserialize,
    Serialize,
};
use std::fmt::{
    Debug,
    Display,
};
use crate::direction::insert::InsertDirection;
use crate::{
    direction::Right,
};
use crate::graph::vertex::token::Tokenize;

pub trait GraphKind: Debug + Clone + Default + PartialEq + Eq {
    type Token: Tokenize + Display + DeserializeOwned;
    type Direction: InsertDirection;
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
