use std::{
    collections::BTreeSet,
    num::NonZeroUsize,
};

use derive_more::derive::{
    Deref,
    DerefMut,
};
use derive_new::new;
use itertools::Itertools;

use super::{
    cache::{
        position::SplitPositionCache,
        vertex::SplitVertexCache,
        PosKey,
    },
    node::RootNode,
    SplitStates,
};
use crate::{
    graph::{
        getters::vertex::VertexSet,
        vertex::{
            child::{
                Child,
                ChildWidth,
            },
            data::VertexData,
            has_vertex_index::HasVertexIndex,
            location::SubLocation,
            wide::Wide,
        },
    },
    interval::{
        IntervalGraph,
        SplitCache,
    },
    path::mutators::move_path::key::TokenPosition,
    traversal::{
        cache::{
            entry::{
                position::{
                    Offset,
                    SubSplitLocation,
                },
                vertex::VertexCache,
            },
            label_key::labelled_key,
        },
        split::{
            cleaned_position_splits,
            node::{
                InnerNode,
                NodeSplitOutput,
                NodeType,
                RootMode,
            },
            position_splits,
        },
        trace::{
            context::{
                node::NodeTraceContext,
                TraceContext,
            },
            TraceState,
        },
        traversable::Traversable,
    },
    HashSet,
};

#[derive(Debug, Copy, Clone, Deref, new)]
pub struct SplitContext<'a> {
    pub cache: &'a VertexCache,
}
impl SplitContext<'_> {
    pub fn global_splits<N: NodeType>(
        &self,
        end_pos: TokenPosition,
        node: &VertexData,
    ) -> N::GlobalSplitOutput {
        let mut output = N::GlobalSplitOutput::default();
        let (mut front, mut back) = (false, false);
        for (inner_width, cache) in &self.bottom_up {
            for location in cache.edges.bottom.values() {
                let child = node.expect_child_at(location);
                let inner_offset = Offset::new(child.width() - inner_width.0);
                let bottom = SubSplitLocation {
                    location: *location,
                    inner_offset,
                };
                let offset = node.expect_child_offset(location);
                if let Some(parent_offset) = inner_offset
                    .and_then(|o| o.checked_add(offset))
                    .or(NonZeroUsize::new(offset))
                {
                    output
                        .splits_mut()
                        .entry(parent_offset)
                        .and_modify(|e: &mut Vec<_>| e.push(bottom.clone()))
                        .or_insert_with(|| vec![bottom]);
                    front = true;
                } else {
                    break;
                }
            }
        }
        for (pretext_pos, cache) in &self.top_down {
            let inner_offset = Offset::new(end_pos.0 - pretext_pos.0).unwrap();
            for location in cache.edges.bottom.values() {
                let child = node.expect_child_at(location);
                let inner_offset = Offset::new(inner_offset.get() % child.width());
                let location = SubLocation {
                    sub_index: location.sub_index + inner_offset.is_none() as usize,
                    pattern_id: location.pattern_id,
                };
                let bottom = SubSplitLocation {
                    location,
                    inner_offset,
                };
                let offset = node.expect_child_offset(&location);
                let parent_offset = inner_offset
                    .map(|o| o.checked_add(offset).unwrap())
                    .unwrap_or_else(|| NonZeroUsize::new(offset).unwrap());
                if parent_offset.get() < node.width {
                    if let Some(e) = output.splits_mut().get_mut(&parent_offset) {
                        e.push(bottom)
                    } else {
                        output.splits_mut().insert(parent_offset, vec![bottom]);
                    }
                    back = true;
                }
            }
        }
        match (front, back) {
            (true, true) => output.set_root_mode(RootMode::Infix),
            (false, true) => output.set_root_mode(RootMode::Prefix),
            (true, false) => output.set_root_mode(RootMode::Postfix),
            (false, false) => unreachable!(),
        }
        output
    }
    pub fn complete_splits<Trav: Traversable, N: NodeType>(
        &self,
        trav: &Trav,
        end_pos: TokenPosition,
    ) -> N::CompleteSplitOutput {
        let graph = trav.graph();

        let node = graph.expect_vertex(self.index);

        let output = self.global_splits::<N>(end_pos, node);

        N::map(output, |global_splits| {
            global_splits
                .into_iter()
                .map(|(parent_offset, mut locs)| {
                    if locs.len() < node.children.len() {
                        let pids: HashSet<_> = locs.iter().map(|l| l.location.pattern_id).collect();
                        let missing = node
                            .children
                            .iter()
                            .filter(|(pid, _)| !pids.contains(pid))
                            .collect_vec();
                        let new_splits = position_splits(missing, parent_offset).splits;
                        locs.extend(new_splits.into_iter().map(|(pid, loc)| SubSplitLocation {
                            location: SubLocation::new(pid, loc.sub_index),
                            inner_offset: loc.inner_offset,
                        }))
                    }
                    (
                        parent_offset,
                        locs.into_iter()
                            .map(|sub| {
                                if sub.inner_offset.is_some()
                                    || node.children[&sub.location.pattern_id].len() > 2
                                {
                                    // can't be clean
                                    Ok(sub)
                                } else {
                                    // must be clean
                                    Err(sub.location)
                                }
                            })
                            .collect(),
                    )
                })
                .collect()
        })
    }
}

#[derive(Debug, Deref, DerefMut)]
pub struct SplitTraceContext<Trav: Traversable> {
    pub root: Child,
    pub end_bound: usize,

    #[deref]
    #[deref_mut]
    pub ctx: TraceContext<Trav>,
}

impl<Trav: Traversable> SplitTraceContext<Trav> {
    pub fn completed_splits<N: NodeType>(
        &self,
        index: &Child,
    ) -> N::CompleteSplitOutput {
        self.cache
            .entries
            .get(&index.vertex_index())
            .map(|e| {
                SplitContext::new(e).complete_splits::<_, N>(&self.trav, self.end_bound.into())
            })
            .unwrap_or_default()
    }
}

#[derive(Debug, Deref, DerefMut)]
pub struct SplitCacheContext<Trav: Traversable> {
    #[deref]
    #[deref_mut]
    pub states_ctx: SplitTraceStateContext<Trav>,

    pub cache: SplitCache,
}
impl<Trav: Traversable> SplitCacheContext<Trav> {
    pub fn init(mut states_ctx: SplitTraceStateContext<Trav>) -> Self {
        let (offsets, root_mode) = states_ctx.completed_splits::<RootNode>(&states_ctx.ctx.root);
        let pos_splits = states_ctx
            .states
            .leaves
            .collect_leaves(&states_ctx.ctx.root, offsets.clone());
        states_ctx.states.filter_trace_states(
            &states_ctx.ctx.trav,
            &states_ctx.ctx.root,
            pos_splits,
        );
        let root_vertex = SplitVertexCache {
            positions: offsets
                .into_iter()
                .map(|(offset, res)| {
                    (
                        offset,
                        SplitPositionCache::root(res.unwrap_or_else(|location| {
                            vec![SubSplitLocation {
                                location,
                                inner_offset: None,
                            }]
                        })),
                    )
                })
                .collect(),
        };
        let cache = SplitCache::new(
            root_mode,
            FromIterator::from_iter([(
                labelled_key(&states_ctx.ctx.trav, states_ctx.ctx.root),
                root_vertex,
            )]),
        );

        Self { states_ctx, cache }
    }
    pub fn augment_node(
        &mut self,
        index: Child,
    ) -> Vec<TraceState> {
        let graph = self.states_ctx.ctx.trav.graph();
        let ctx = NodeTraceContext::new(&graph, index);
        self.cache
            .get_mut(&index.vertex_index())
            .unwrap()
            .augment_node(ctx)
    }
    /// complete inner range offsets for root
    pub fn augment_root(&mut self) -> Vec<TraceState> {
        let graph = self.states_ctx.ctx.trav.graph();
        let ctx = NodeTraceContext::new(&graph, self.root);
        let index = self.root.vertex_index();
        let root_mode = self.cache.root_mode;
        self.cache
            .get_mut(&index)
            .unwrap()
            .augment_root(ctx, root_mode)
    }
    pub fn augment_nodes<I: IntoIterator<Item = Child>>(
        &mut self,
        nodes: I,
    ) {
        for c in nodes {
            let new = self.augment_node(c);
            // todo: force order
            self.states.queue.extend(new.into_iter());
        }
    }
    fn apply_trace_state(
        &mut self,
        state: &TraceState,
    ) {
        let &TraceState {
            index,
            offset,
            prev,
        } = state;
        if let Some(ve) = self.cache.get_mut(&index.vertex_index()) {
            ve.positions
                .entry(offset)
                .and_modify(|pe| {
                    pe.top.insert(prev);
                })
                .or_insert_with(|| self.states_ctx.new_split_position(index, offset, prev));
        } else {
            let vertex = self.new_split_vertex(index, offset, prev);
            self.cache
                .insert(labelled_key(&self.states_ctx.ctx.trav, index), vertex);
        }
    }
}

#[derive(Debug)]
pub struct SplitRunStep;

#[derive(Debug)]
pub struct SplitRun<Trav: Traversable> {
    cache: SplitCacheContext<Trav>,
    incomplete: BTreeSet<Child>,
}
impl<'a, Trav: Traversable + 'a> SplitRun<Trav> {
    pub fn init(&mut self) {
        self.cache.augment_root();
    }
    pub fn finish(mut self) -> SplitCacheContext<Trav> {
        self.cache.augment_nodes(self.incomplete);
        self.cache
    }
}
impl<'a, Trav: Traversable + 'a> Iterator for SplitRun<Trav> {
    type Item = SplitRunStep;
    fn next(&mut self) -> Option<Self::Item> {
        self.cache.states_ctx.states.next().map(|state| {
            self.cache.apply_trace_state(&state);
            self.incomplete.insert(state.index);
            let complete = self
                .incomplete
                .split_off(&ChildWidth(state.index.width() + 1));
            self.cache.augment_nodes(complete);
            SplitRunStep
        })
    }
}
impl<'a, Trav: Traversable + 'a> From<SplitCacheContext<Trav>> for SplitRun<Trav> {
    fn from(cache: SplitCacheContext<Trav>) -> Self {
        Self {
            cache,
            incomplete: Default::default(),
        }
    }
}
impl<'a, Trav: Traversable + 'a> From<SplitCacheContext<Trav>> for IntervalGraph {
    fn from(cache: SplitCacheContext<Trav>) -> Self {
        Self::from(SplitRun::from(cache))
    }
}
impl<'a, Trav: Traversable + 'a> From<SplitRun<Trav>> for IntervalGraph {
    fn from(mut run: SplitRun<Trav>) -> Self {
        run.init();
        run.all(|_| true); // run iterator to end
        let cache = run.finish();
        Self {
            root: cache.root,
            states: cache.states_ctx.states,
            cache: cache.cache,
        }
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
