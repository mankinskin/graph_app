use std::fmt::Debug;

pub mod r#match;
pub mod pattern;
//pub(crate) mod merge;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Left;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Right;

pub trait Direction: Clone + Debug + Send + Sync + 'static + Unpin {
    type Opposite: Direction;
}

impl Direction for Left {
    type Opposite = Right;
}

impl Direction for Right {
    type Opposite = Left;
}

pub struct Both;
