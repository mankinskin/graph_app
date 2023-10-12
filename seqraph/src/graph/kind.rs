use crate::*;

pub trait GraphKind: Debug + Clone + Default {
    type Token: Tokenize + Display;
    type Direction: IndexDirection;
}
pub type TokenOf<K> = <K as GraphKind>::Token;

#[derive(Debug, Clone, Default)]
pub struct BaseGraphKind;

impl GraphKind for BaseGraphKind {
    type Token = char;
    type Direction = Right;
}