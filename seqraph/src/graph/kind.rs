use crate::shared::*;

pub trait GraphKind: Debug + Clone + Default {
    type Token: Tokenize + Display;
    type Direction: IndexDirection;
}
pub type TokenOf<K> = <K as GraphKind>::Token;
pub type DefaultToken = TokenOf<BaseGraphKind>;
pub type DirectionOf<K> = <K as GraphKind>::Direction;
pub type DefaultDirection = DirectionOf<BaseGraphKind>;

#[derive(Debug, Clone, Default)]
pub struct BaseGraphKind;

impl GraphKind for BaseGraphKind {
    type Token = char;
    type Direction = Right;
}