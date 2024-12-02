pub mod insert;
pub mod r#match;
pub mod merge;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Left;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Right;

pub trait Direction {
    type Opposite: Direction;
}

impl Direction for Left {
    type Opposite = Right;
}

impl Direction for Right {
    type Opposite = Left;
}

pub struct Both;