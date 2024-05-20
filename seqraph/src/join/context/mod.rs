pub mod node;

use node::*;
use std::sync::RwLockWriteGuard;

pub mod pattern;
use crate::{
    join::partition::splits::{
        HasSubSplits,
        SubSplits,
    },
    split::SplitCache,
    vertex::{
        child::Child,
        indexed::Indexed,
    },
};

#[derive(Debug)]
pub struct JoinContext<'p> {
    pub graph: RwLockWriteGuard<'p, crate::graph::Hypergraph>,
    pub sub_splits: &'p SubSplits,
}
impl<'p> JoinContext<'p> {
    pub fn new<SS: HasSubSplits>(
        graph: RwLockWriteGuard<'p, crate::graph::Hypergraph>,
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
    ) -> NodeJoinContext {
        NodeJoinContext::new(
            self,
            index,
            split_cache.entries.get(&index.vertex_index()).unwrap(),
        )
    }
}
// , PatternCtx<'p> = PatternJoinContext<'p>
pub trait AsNodeJoinContext<'p> {
    fn as_node_join_context<'t>(self) -> NodeJoinContext<'t>
    where
        Self: 't,
        'p: 't;
}
impl<'p> AsNodeJoinContext<'p> for NodeJoinContext<'p> {
    fn as_node_join_context<'t>(self) -> NodeJoinContext<'t>
    where
        Self: 't,
        'p: 't,
    {
        self
    }
}
