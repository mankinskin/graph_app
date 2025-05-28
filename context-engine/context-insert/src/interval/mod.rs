use std::fmt::Debug;

use crate::split::{
    cache::{
        SplitCache,
        position::{
            PosKey,
            SplitPositionCache,
        },
    },
    trace::states::SplitStates,
};
use context_trace::graph::vertex::{
    child::Child,
    has_vertex_index::HasVertexIndex,
};

pub mod init;
pub mod partition;

#[derive(Debug)]
pub struct IntervalGraph {
    pub states: SplitStates,
    pub cache: SplitCache,
    pub root: Child,
}
impl IntervalGraph {
    pub fn get(
        &self,
        key: &PosKey,
    ) -> Option<&SplitPositionCache> {
        self.cache
            .get(&key.index.vertex_index())
            .and_then(|ve| ve.positions.get(&key.pos))
    }
    pub fn get_mut(
        &mut self,
        key: &PosKey,
    ) -> Option<&mut SplitPositionCache> {
        self.cache
            .get_mut(&key.index.vertex_index())
            .and_then(|ve| ve.positions.get_mut(&key.pos))
    }
    pub fn expect(
        &self,
        key: &PosKey,
    ) -> &SplitPositionCache {
        self.get(key).unwrap()
    }
    pub fn expect_mut(
        &mut self,
        key: &PosKey,
    ) -> &mut SplitPositionCache {
        self.get_mut(key).unwrap()
    }
}
