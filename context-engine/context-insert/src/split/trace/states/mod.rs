use std::collections::VecDeque;

use super::SplitTraceCtx;
use crate::split::{
    cache::{
        leaves::Leaves,
        position::PosKey,
    },
    trace::{
        HasGraph,
        SplitTraceState,
    },
};
use context_trace::*;
pub mod context;
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct SplitStates {
    pub leaves: Leaves,
    pub queue: VecDeque<SplitTraceState>,
}
impl Iterator for SplitStates {
    type Item = SplitTraceState;
    fn next(&mut self) -> Option<Self::Item> {
        self.queue.pop_front()
    }
}
impl SplitStates {
    /// kind of like filter_leaves but from subsplits to trace states
    pub fn filter_trace_states<G: HasGraph>(
        &mut self,
        trav: G,
        index: &Child,
        pos_splits: impl IntoIterator<Item = (Offset, Vec<SubSplitLocation>)>,
    ) {
        let (perfect, next) = {
            let graph = trav.graph();
            let node = graph.expect_vertex(index);
            pos_splits
                .into_iter()
                .flat_map(|(parent_offset, locs)| {
                    let len = locs.len();
                    locs.into_iter().map(move |sub|
                    // filter sub locations without offset (perfect splits)
                    sub.inner_offset.map(|offset|
                        SplitTraceState {
                            index: *node.expect_child_at(&sub.location),
                            offset,
                            prev: PosKey {
                                index: *index,
                                pos: parent_offset,
                            },
                        }
                    ).ok_or_else(||
                        (len == 1).then(||
                            PosKey::new(*index, parent_offset)
                        )
                    ))
                })
                .fold((Vec::new(), Vec::new()), |(mut p, mut n), res| {
                    match res {
                        Ok(s) => n.push(s),
                        Err(Some(k)) => p.push(k),
                        Err(None) => {},
                    }
                    (p, n)
                })
        };
        self.leaves.extend(perfect);
        self.queue.extend(next);
    }
}
