use crate::traversal::ParentBatch;

use super::{
    top_down::end::EndState,
    StateNext,
};

pub mod parent;
pub(crate) mod start;

#[derive(Clone, Debug)]
pub enum BUNext {
    Parents(StateNext<ParentBatch>),
    End(StateNext<EndState>),
}
