use crate::{
    graph::Hypergraph,
    partition::splits::HasPosSplits,
    split::cache::vertex::SplitVertexCache,
    traversal::traversable::TraversableMut,
};
use std::fmt::Debug;

pub trait JoinKind: Debug + Clone + Copy{
    type Trav: TraversableMut;
    type SP: HasPosSplits + Debug;
}
#[derive(Debug, Clone, Copy)]
pub struct DefaultJoin;

impl JoinKind for DefaultJoin {
    type Trav = Hypergraph;
    type SP = SplitVertexCache;
}
