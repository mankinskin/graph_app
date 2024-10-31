use std::collections::VecDeque;

use itertools::Itertools;

use crate::graph::{
    traversal::pass::TraversalPass,
    vocabulary::{
        entry::VertexCtx,
        NGramId,
        Vocabulary,
    },
};
use seqraph::{
    graph::vertex::{
        child::Child,
        key::VertexKey,
        wide::Wide,
        VertexIndex,
    },
    HashSet,
};

pub trait Visited<P: TraversalPass> {
    fn insert(&mut self, node: <P as TraversalPass>::Node) -> bool;
}

impl<P: TraversalPass> Visited<P> for HashSet<P::Node> {
    fn insert(&mut self, node: <P as TraversalPass>::Node) -> bool {
        HashSet::insert(self, node)
    }
}
impl<P: TraversalPass> Visited<P> for () {
    fn insert(&mut self, node: <P as TraversalPass>::Node) -> bool {
        true
    }
}