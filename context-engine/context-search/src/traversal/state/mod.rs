pub mod cursor;
pub(crate) mod traversal;

pub mod end;
pub mod start;

use crate::traversal::{
    compare::state::CompareState,
    EndState,
};

#[derive(Clone, Debug)]
pub struct StateNext<T> {
    //pub prev: PrevKey,
    pub inner: T,
}

#[derive(Clone, Debug)]
pub enum ChildMatchState {
    Mismatch(EndState),
    Match(CompareState),
}
