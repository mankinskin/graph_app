use crate::{
    graph::vertex::{
        child::Child,
        has_vertex_index::HasVertexIndex,
    },
    trace::has_graph::{
        HasGraph,
        TravToken,
    },
};
use std::fmt::Display;

pub type Labelled<T> = T;

pub fn labelled_key<G: HasGraph, T>(
    _trav: &G,
    index: T,
) -> Labelled<T>
where
    TravToken<G>: Display,
{
    index
}

#[macro_export]
macro_rules! lab {
    ($x:ident) => {
        $x.vertex_index()
    };
}
