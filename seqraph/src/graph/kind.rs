use crate::*;

pub trait GraphKind: Debug + Clone + Default {
    type Token: Tokenize;
    type Direction: IndexDirection;
}

#[derive(Debug, Clone, Default)]
pub struct BaseGraphKind;

impl GraphKind for BaseGraphKind {
    type Token = char;
    type Direction = Right;
}