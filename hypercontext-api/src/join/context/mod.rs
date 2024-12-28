use std::sync::RwLockWriteGuard;

use crate::{
    join::{
        context::node::context::NodeJoinContext,
        partition::splits::{
            HasSubSplits,
            SubSplits,
        },
    },
    split::cache::SplitCache,
};
use hypercontext_api::graph::{
    Hypergraph,
    vertex::{
        child::Child,
        has_vertex_index::HasVertexIndex,
    },
};

pub mod node;

pub mod pattern;

#[derive(Debug)]
pub struct JoinContext<'p> {
    pub graph: RwLockWriteGuard<'p, Hypergraph>,
    pub sub_splits: &'p SubSplits,
}

impl<'p> JoinContext<'p> {
    pub fn new<SS: HasSubSplits>(
        graph: RwLockWriteGuard<'p, Hypergraph>,
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
pub trait ToNodeJoinContext<'p> {
    fn to_node_join_context<'t>(self) -> NodeJoinContext<'t>
    where
        Self: 't,
        'p: 't;
}

impl<'p> ToNodeJoinContext<'p> for NodeJoinContext<'p> {
    fn to_node_join_context<'t>(self) -> NodeJoinContext<'t>
    where
        Self: 't,
        'p: 't,
    {
        self
    }
}
