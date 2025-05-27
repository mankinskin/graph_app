use std::num::NonZeroUsize;

use derive_more::derive::{
    Deref,
    DerefMut,
};

use super::{
    SplitStates,
    SplitTraceContext,
};
use crate::split::{
    cache::{
        position::{
            PosKey,
            SplitPositionCache,
        },
        vertex::SplitVertexCache,
    },
    cleaned_position_splits,
    trace::HasGraph,
    vertex::output::InnerNode,
};
use context_trace::{
    graph::{
        getters::vertex::VertexSet,
        vertex::child::Child,
    },
    trace::{
        TraceContext,
        cache::position::SubSplitLocation,
    },
};

#[derive(Debug, Deref, DerefMut)]
pub struct SplitTraceStatesContext<G: HasGraph> {
    #[deref]
    #[deref_mut]
    pub ctx: SplitTraceContext<G>,
    pub states: SplitStates,
}
impl<'a, G: HasGraph> SplitTraceStatesContext<G> {
    pub fn new(
        ctx: TraceContext<G>,
        root: Child,
        end_bound: usize,
    ) -> Self {
        Self {
            ctx: SplitTraceContext {
                ctx,
                root,
                end_bound,
            },
            states: SplitStates::default(),
        }
    }
    pub fn new_split_vertex(
        &mut self,
        index: Child,
        offset: NonZeroUsize,
        prev: PosKey,
    ) -> SplitVertexCache {
        let mut subs = self.completed_splits::<InnerNode>(&index);
        subs.entry(offset).or_insert_with(|| {
            let graph = self.ctx.trav.graph();
            let node = graph.expect_vertex(index);
            //let entry = self.cache.entries.get(&index.index).unwrap();
            cleaned_position_splits(node.children.iter(), offset)
        });
        let pos_splits =
            self.states.leaves.collect_leaves(&index, subs.clone());
        self.states
            .filter_trace_states(&self.ctx.trav, &index, pos_splits);
        SplitVertexCache {
            positions: subs
                .into_iter()
                .map(|(offset, res)| {
                    (
                        offset,
                        SplitPositionCache::new(
                            prev,
                            res.unwrap_or_else(|location| {
                                vec![SubSplitLocation {
                                    location,
                                    inner_offset: None,
                                }]
                            }),
                        ),
                    )
                })
                .collect(),
        }
    }
    pub fn new_split_position(
        &mut self,
        index: Child,
        offset: NonZeroUsize,
        prev: PosKey,
    ) -> SplitPositionCache {
        let splits = {
            let graph = self.ctx.trav.graph();
            let node = graph.expect_vertex(index);
            cleaned_position_splits(node.children.iter(), offset)
        };

        // handle clean splits
        match splits {
            Ok(subs) => {
                self.states.filter_trace_states(
                    &self.ctx.trav,
                    &index,
                    Vec::from_iter([(offset, subs.clone())]),
                );
                SplitPositionCache::new(prev, subs)
            },
            Err(location) => {
                self.states.leaves.push(PosKey::new(index, offset));
                SplitPositionCache::new(prev, vec![SubSplitLocation {
                    location,
                    inner_offset: None,
                }])
            },
        }
    }
}
