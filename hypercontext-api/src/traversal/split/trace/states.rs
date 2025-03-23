use std::{
    collections::VecDeque,
    num::NonZeroUsize,
};

use derive_more::derive::{
    Deref,
    DerefMut,
};

use crate::{
    graph::{
        getters::vertex::VertexSet,
        vertex::child::Child,
    },
    traversal::{
        cache::entry::position::{
            Offset,
            SubSplitLocation,
        },
        split::{
            cache::{
                leaves::Leaves,
                position::{
                    PosKey,
                    SplitPositionCache,
                },
                vertex::SplitVertexCache,
            },
            cleaned_position_splits,
            trace::{
                context::TraceContext,
                SplitTraceState,
            },
            vertex::output::InnerNode,
        },
        traversable::Traversable,
    },
};

use super::SplitTraceContext;

#[derive(Debug, Default)]
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
    pub fn filter_trace_states<Trav: Traversable>(
        &mut self,
        trav: Trav,
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
                        Err(None) => {}
                    }
                    (p, n)
                })
        };
        self.leaves.extend(perfect);
        self.queue.extend(next);
    }
}

#[derive(Debug, Deref, DerefMut)]
pub struct SplitTraceStateContext<Trav: Traversable> {
    #[deref]
    #[deref_mut]
    pub ctx: SplitTraceContext<Trav>,
    pub states: SplitStates,
}
impl<Trav: Traversable> SplitTraceStateContext<Trav> {
    pub fn new(
        ctx: TraceContext<Trav>,
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
        let pos_splits = self.states.leaves.collect_leaves(&index, subs.clone());
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
            }
            Err(location) => {
                self.states.leaves.push(PosKey::new(index, offset));
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
