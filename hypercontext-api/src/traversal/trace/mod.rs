use crate::graph::vertex::child::Child;

use super::{
    cache::entry::position::Offset,
    split::cache::PosKey,
};

pub(crate) mod context;
pub(crate) mod traceable;

#[derive(Debug, Clone)]
pub struct TraceState {
    pub index: Child,
    pub offset: Offset,
    pub prev: PosKey,
}
