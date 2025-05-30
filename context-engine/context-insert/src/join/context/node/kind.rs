use crate::{
    interval::partition::info::range::role::{
        In,
        Post,
        Pre,
        RangeRole,
    },
    join::partition::Join,
};
use context_trace::{
    graph::Hypergraph,
    trace::has_graph::HasGraphMut,
};
use std::fmt::Debug;

pub trait JoinKind: RangeRole<Mode = Join> + Debug + Clone + Copy {
    type Trav: HasGraphMut;
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
