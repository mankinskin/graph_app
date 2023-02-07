use crate::*;

pub mod path;
pub use path::*;

pub mod end;
pub use end::*;

pub mod query;
pub use query::*;
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NodeDirection {
    BottomUp,
    TopDown,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WaitingState {
    pub prev: CacheKey,
    pub matched: bool,
    pub state: ParentState,
    //pub query: QueryState,
}
