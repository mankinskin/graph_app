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

use super::pass::PassNode;

pub trait VisitTracking: TraversalPass {
    type Collection: VisitorCollection<Self::Node>;
    fn visited_mut(&mut self) -> &mut Self::Collection;
}
pub trait VisitorCollection<N: PassNode> {
    type Ref<'t>: VisitorCollection<N> where N: 't;
    fn insert(&mut self, node: N) -> bool;
}

impl<N: PassNode> VisitorCollection<N> for HashSet<N>
{
    type Ref<'t> = &'t mut Self where N: 't;
    fn insert(&mut self, node: N) -> bool {
        <&mut Self as VisitorCollection<N>>::insert(&mut &mut *self, node)
    }
}
impl<N: PassNode> VisitorCollection<N> for &'_ mut HashSet<N> {
    type Ref<'t> = &'t mut HashSet<N> where N: 't;
    fn insert(&mut self, node: N) -> bool {
        HashSet::insert(*self, node)
    }
}
impl<N: PassNode> VisitorCollection<N> for () {
    type Ref<'t> = Self where N: 't;
    fn insert(&mut self, node: N) -> bool {
        true
    }
}