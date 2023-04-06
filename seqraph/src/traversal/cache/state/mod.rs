use crate::*;

pub mod traversal;
pub use traversal::*;

pub mod end;
pub use end::*;

pub mod query;
pub use query::*;
#[derive(Clone, Debug, PartialEq, Eq, Copy, Hash)]
pub enum NodeDirection {
    BottomUp,
    TopDown,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WaitingState {
    pub prev: DirectedKey,
    pub state: ParentState,
}
