use crate::{
    graph::Hypergraph,
    join::partition::Join,
    interval::partition::info::range::role::{
        In,
        Post,
        Pre,
        RangeRole,
    },
    traversal::traversable::TraversableMut,
};
use std::fmt::Debug;

pub trait JoinKind: RangeRole<Mode = Join> + Debug + Clone + Copy {
    type Trav: TraversableMut;
}

impl JoinKind for Pre<Join> {
    type Trav = Hypergraph;
}
impl JoinKind for In<Join> {
    type Trav = Hypergraph;
}

impl JoinKind for Post<Join> {
    type Trav = Hypergraph;
}
