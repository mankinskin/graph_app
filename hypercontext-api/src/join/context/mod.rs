use std::sync::RwLockWriteGuard;

use node::kind::{DefaultJoin, JoinKind};

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


#[derive(Debug)]
pub struct JoinContext<'p, K: JoinKind + 'p = DefaultJoin> {
    pub graph: <K::Trav as TraversableMut>::GuardMut<'p>,
    pub sub_splits: &'p SubSplits,
}

impl<'p, K: JoinKind> JoinContext<'p, K> {
    pub fn new<SS: HasSubSplits>(
        graph: <<K as JoinKind>::Trav as TraversableMut>::GuardMut<'p>,
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
    ) -> NodeJoinContext<'p, K> {
        NodeJoinContext::new(
            self,
            index,
            split_cache.entries.get(&index.vertex_index()).unwrap(),
        )
    }
}

// , PatternCtx<'p> = PatternJoinContext<'p>

