use crate::interval::IntervalGraph;
use context_trace::{
    graph::{
        Hypergraph,
        HypergraphRef,
        vertex::child::Child,
    },
    trace::has_graph::{
        HasGraph,
        HasGraphMut,
        TravKind,
    },
};
use frontier::FrontierSplitIterator;

pub mod frontier;
pub mod node;
pub mod pattern;

#[derive(Debug)]
pub struct JoinContext {
    pub trav: HypergraphRef,
    pub interval: IntervalGraph,
}
impl JoinContext {
    pub fn join_subgraph(self) -> Child {
        FrontierSplitIterator::from((self.trav, self.interval))
            .find_map(|joined| joined)
            .unwrap()
    }
}

impl HasGraph for JoinContext {
    type Kind = TravKind<Hypergraph>;
    type Guard<'g>
        = <HypergraphRef as HasGraph>::Guard<'g>
    where
        Self: 'g;
    fn graph(&self) -> Self::Guard<'_> {
        self.trav.graph()
    }
}
impl HasGraphMut for JoinContext {
    type GuardMut<'g>
        = <HypergraphRef as HasGraphMut>::GuardMut<'g>
    where
        Self: 'g;
    fn graph_mut(&mut self) -> Self::GuardMut<'_> {
        self.trav.graph_mut()
    }
}
