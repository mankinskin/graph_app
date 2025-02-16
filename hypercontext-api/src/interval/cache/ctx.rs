use std::{
    collections::VecDeque,
    iter::FromIterator,
    num::NonZeroUsize,
};

use crate::{
    graph::{
        getters::vertex::VertexSet,
        vertex::child::Child,
    },
    interval::cache::{
        cleaned_position_splits,
        leaves::Leaves,
        position::SplitPositionCache,
        TraceState,
    },
    traversal::{
        cache::entry::position::SubSplitLocation,
        traversable::Traversable,
    },
};

use super::PosKey;

#[derive(Debug)]
pub struct TraceContext {
    pub leaves: Leaves,
    pub states: VecDeque<TraceState>,
}

impl TraceContext {
    pub fn new_split_position<Trav: Traversable>(
        &mut self,
        trav: &Trav,
        index: Child,
        offset: NonZeroUsize,
        prev: PosKey,
    ) -> SplitPositionCache {
        let graph = trav.graph();
        let node = graph.expect_vertex(index);

        // handle clean splits
        match cleaned_position_splits(node.children.iter(), offset) {
            Ok(subs) => {
                let next = self.leaves.filter_trace_states(
                    trav,
                    &index,
                    Vec::from_iter([(offset, subs.clone())]),
                );
                self.states.extend(next);
                SplitPositionCache::new(prev, subs)
            }
            Err(location) => {
                self.leaves.push(PosKey::new(index, offset));
                SplitPositionCache::new(
                    prev,
                    vec![SubSplitLocation {
                        location,
                        inner_offset: None,
                    }],
                )
            }
        }
    }
}
