use std::{
    collections::{
        BTreeSet,
        VecDeque,
    },
    num::NonZeroUsize,
};

use crate::{
    graph::{
        getters::vertex::VertexSet,
        kind::GraphKind,
        vertex::{
            child::{
                Child,
                ChildWidth,
            },
            has_vertex_index::HasVertexIndex,
            wide::Wide,
        },
        Hypergraph,
    },
    partition::context::NodeTraceContext,
    split::{
        cache::{
            cleaned_position_splits,
            leaves::Leaves,
            vertex::SplitVertexCache,
            CacheContext,
            SplitCache,
            SplitPositionCache,
            TraceState,
        },
        SplitContext,
    },
    traversal::{
        cache::{
            entry::{
                position::SubSplitLocation,
                InnerNode,
                NodeType,
                RootMode,
                RootNode,
            },
            key::SplitKey,
            label_key::vkey::labelled_key,
        },
        fold::state::FoldState,
        traversable::{
            Traversable,
            TraversableMut,
        },
    },
    HashMap,
};
use derive_more::{
    Deref,
    DerefMut,
};

#[derive(Debug, Deref, DerefMut)]
pub struct SplitCacheBuilder(pub SplitCache);

impl SplitCacheBuilder {
    pub fn new<'a, Trav: TraversableMut + 'a>(
        trav: &'a mut Trav,
        mut fold_state: &mut FoldState,
    ) -> Self {
        let mut entries = HashMap::default();

        let (root_vertex, root_mode, states, leaves) = Self::new_root_vertex(trav, &fold_state);
        entries.insert(labelled_key(trav, fold_state.root), root_vertex);
        let mut cache = Self(SplitCache {
            entries,
            root_mode,
            root: fold_state.root,
            context: CacheContext { leaves, states },
        });
        let graph = trav.graph_mut();
        cache.augment_root(NodeTraceContext::new(&*graph, fold_state.root), root_mode);
        // stores past states
        let mut incomplete = BTreeSet::<Child>::default();
        // traverse top down by width
        // cache indices without range infos
        // calc range infos for larger indices when smaller index is traversed
        while let Some(state) = cache.states.pop_front() {
            // trace offset splits top down by width
            // complete past states larger than current state
            // store offsets and filter leaves
            cache.trace(&graph, &mut fold_state, &state);
            incomplete.insert(state.index);
            let complete = incomplete.split_off(&ChildWidth(state.index.width() + 1));
            cache.augment_nodes(&*graph, complete);
        }
        cache.augment_nodes(&*graph, incomplete);
        cache
    }
    pub fn new_root_vertex<Trav: Traversable>(
        trav: &Trav,
        fold_state: &FoldState,
    ) -> (SplitVertexCache, RootMode, VecDeque<TraceState>, Leaves) {
        let mut states = VecDeque::default();
        let mut leaves = Leaves::default();
        let (offsets, root_mode) =
            Self::completed_splits::<_, RootNode>(trav, fold_state, &fold_state.root);
        let pos_splits = leaves.filter_leaves(&fold_state.root, offsets.clone());
        states.extend(leaves.filter_trace_states(trav, &fold_state.root, pos_splits));
        (
            SplitVertexCache {
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
            },
            root_mode,
            states,
            leaves,
        )
    }
    pub fn new_split_vertex<Trav: Traversable>(
        &mut self,
        trav: &Trav,
        fold_state: &mut FoldState,
        index: Child,
        offset: NonZeroUsize,
        prev: SplitKey,
    ) -> SplitVertexCache {
        let mut subs = Self::completed_splits::<_, InnerNode>(trav, fold_state, &index);
        subs.entry(offset).or_insert_with(|| {
            let graph = trav.graph();
            let node = graph.expect_vertex(index);
            //let entry = self.cache.entries.get(&index.index).unwrap();
            cleaned_position_splits(node.children.iter(), offset)
        });
        let pos_splits = self.leaves.filter_leaves(&index, subs.clone());
        let next = self.leaves.filter_trace_states(trav, &index, pos_splits);
        self.states.extend(next);
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
    /// complete offsets across all children
    pub fn trace<Trav: Traversable>(
        &mut self,
        trav: &Trav,
        fold_state: &mut FoldState,
        state: &TraceState,
    ) {
        let &TraceState {
            index,
            offset,
            prev,
        } = state;
        if let Some(ve) = self.0.entries.get_mut(&index.vertex_index()) {
            let ctx = &mut self.0.context;
            ve.positions
                .entry(offset)
                .and_modify(|pe| {
                    pe.top.insert(prev);
                })
                .or_insert_with(|| ctx.new_split_position(trav, index, offset, prev));
        } else {
            let vertex = self.new_split_vertex(trav, fold_state, index, offset, prev);
            self.entries.insert(labelled_key(trav, index), vertex);
        }
    }
    pub fn child_trace_states<Trav: Traversable>(
        &mut self,
        trav: &Trav,
        fold_state: &FoldState,
        index: &Child,
    ) -> Vec<TraceState> {
        let subs = Self::completed_splits::<_, InnerNode>(trav, fold_state, index)
            .into_iter()
            .filter_map(|(parent_offset, res)| res.ok().map(|locs| (parent_offset, locs)));
        self.leaves.filter_trace_states(trav, index, subs)
    }
    pub fn build(self) -> SplitCache {
        self.0
    }

    /// complete inner range offsets for non-roots
    pub fn augment_node(
        &mut self,
        ctx: NodeTraceContext,
    ) -> Vec<TraceState> {
        self.entries
            .get_mut(&ctx.index.vertex_index())
            .unwrap()
            .augment_node(ctx)
    }
    /// complete inner range offsets for root
    pub fn augment_root(
        &mut self,
        ctx: NodeTraceContext,
        root_mode: RootMode,
    ) -> Vec<TraceState> {
        self.entries
            .get_mut(&ctx.index.vertex_index())
            .unwrap()
            .augment_root(ctx, root_mode)
    }
    pub fn augment_nodes<K: GraphKind, I: IntoIterator<Item = Child>>(
        &mut self,
        graph: &Hypergraph<K>,
        nodes: I,
    ) {
        for c in nodes {
            let new = self.augment_node(NodeTraceContext::new(graph, c));
            // todo: force order
            self.states.extend(new.into_iter());
        }
    }
    pub fn completed_splits<Trav: Traversable, N: NodeType>(
        trav: &Trav,
        fold_state: &FoldState,
        index: &Child,
    ) -> N::CompleteSplitOutput {
        fold_state
            .cache
            .entries
            .get(&index.vertex_index())
            .map(|e| {
                SplitContext::new(e)
                    .complete_splits::<_, N>(trav, fold_state.end_state.width().into())
            })
            .unwrap_or_default()
    }
}
