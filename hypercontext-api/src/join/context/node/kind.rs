use crate::{
    graph::Hypergraph, join::partition::Join, partition::{info::range::role::{In, Post, Pre, RangeRole}, splits::HasPosSplits}, split::cache::vertex::SplitVertexCache, traversal::traversable::TraversableMut
};
use std::fmt::Debug;

pub trait JoinKind: RangeRole<Mode = Join> + Debug + Clone + Copy {
    type Trav: TraversableMut;
    type SP: HasPosSplits + Debug;
}

impl JoinKind for Pre<Join> {
    type Trav = Hypergraph;
    type SP = SplitVertexCache;
}
impl JoinKind for In<Join> {
    type Trav = Hypergraph;
    type SP = SplitVertexCache;
}

impl JoinKind for Post<Join> {
    type Trav = Hypergraph;
    type SP = SplitVertexCache;
}