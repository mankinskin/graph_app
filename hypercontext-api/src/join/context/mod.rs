use std::sync::RwLockWriteGuard;

use node::kind::JoinKind;

use crate::graph::Hypergraph;
use crate::partition::splits::HasSubSplits;
use crate::traversal::traversable::TraversableMut;
use crate::{
    join::context::node::context::NodeJoinContext, partition::splits::SubSplits, split::cache::SplitCache
};
use crate::graph::vertex::{
        child::Child,
        has_vertex_index::HasVertexIndex,
    };

pub mod node;
pub mod pattern;


#[derive(Debug)]
pub struct JoinContext<'p> {
    pub graph: <Hypergraph as TraversableMut>::GuardMut<'p>,
    pub sub_splits: &'p SubSplits,
}

impl<'p> JoinContext<'p> {
    pub fn new<SS: HasSubSplits>(
        graph: <Hypergraph as TraversableMut>::GuardMut<'p>,
        sub_splits: &'p SS,
    ) -> Self {
        Self {
            graph,
            sub_splits: sub_splits.sub_splits(),
        }
    }
    pub fn node(
        self,
        index: Child,
        split_cache: &'p SplitCache,
    ) -> NodeJoinContext<'p> {
        NodeJoinContext::new(
            self,
            index,
            split_cache.entries.get(&index.vertex_index()).unwrap(),
        )
    }
}

// , PatternCtx<'p> = PatternJoinContext<'p>

