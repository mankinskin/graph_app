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
    interval::{
        cache::{
            cleaned_position_splits,
            leaves::Leaves,
            vertex::SplitVertexCache,
            PosKey,
            TraceState,
        },
        partition::context::NodeTraceContext,
        split::SplitContext,
        SplitPositionCache,
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
            label_key::vkey::labelled_key,
            TraversalCache,
        },
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

use super::{
    cache::ctx::TraceContext,
    InitInterval,
    IntervalGraph,
    SplitVertices,
};

#[derive(Debug, Deref, DerefMut)]
pub struct IntervalGraphBuilder {
    vertices: SplitVertices,
    root_mode: RootMode,
    root: Child,
    end_bound: usize,
    #[deref]
    #[deref_mut]
    ctx: TraceContext,
    cache: TraversalCache,
}

impl IntervalGraphBuilder {
    pub fn new<'a, Trav: TraversableMut + 'a>(
        trav: &'a mut Trav,
        init: InitInterval,
    ) -> Self {
        let InitInterval {
            cache,
            root,
            end_bound,
        } = init;
        let mut entries = HashMap::default();
        let mut states = VecDeque::default();
        let mut leaves = Leaves::default();
        // create root vertex
        let (offsets, root_mode) =
            Self::completed_splits::<_, RootNode>(trav, &cache, &root, end_bound);
        let pos_splits = leaves.filter_leaves(&root, offsets.clone());
        states.extend(leaves.filter_trace_states(trav, &root, pos_splits));

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

        entries.insert(labelled_key(trav, root), root_vertex);
        Self {
            vertices: SplitVertices { entries },
            root_mode,
            end_bound,
            root,
            ctx: TraceContext { leaves, states },
            cache,
        }
        .init(trav)
    }
    fn init<'a, Trav: TraversableMut + 'a>(
        mut self,
        trav: &'a mut Trav,
    ) -> Self {
        let graph = trav.graph_mut();
        self.augment_root(NodeTraceContext::new(&*graph, self.root), self.root_mode);
        // stores past states
        let mut incomplete = BTreeSet::<Child>::default();
        // traverse top down by width
        // cache indices without range infos
        // calc range infos for larger indices when smaller index is traversed
        while let Some(state) = self.states.pop_front() {
            // trace offset splits top down by width
            // complete past states larger than current state
            // store offsets and filter leaves
            self.trace(&graph, &state);
            incomplete.insert(state.index);
            let complete = incomplete.split_off(&ChildWidth(state.index.width() + 1));
            self.augment_nodes(&*graph, complete);
        }
        self.augment_nodes(&*graph, incomplete);
        self
    }
    pub fn new_split_vertex<Trav: Traversable>(
        &mut self,
        trav: &Trav,
        index: Child,
        offset: NonZeroUsize,
        prev: PosKey,
    ) -> SplitVertexCache {
        let mut subs =
            Self::completed_splits::<_, InnerNode>(trav, &self.cache, &index, self.end_bound);
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
        state: &TraceState,
    ) {
        let &TraceState {
            index,
            offset,
            prev,
        } = state;
        if let Some(ve) = self.vertices.get_mut(&index.vertex_index()) {
            let ctx = &mut self.ctx;
            ve.positions
                .entry(offset)
                .and_modify(|pe| {
                    pe.top.insert(prev);
                })
                .or_insert_with(|| ctx.new_split_position(trav, index, offset, prev));
        } else {
            let vertex = self.new_split_vertex(trav, index, offset, prev);
            self.vertices.insert(labelled_key(trav, index), vertex);
        }
    }
    pub fn child_trace_states<Trav: Traversable>(
        &mut self,
        trav: &Trav,
        cache: &TraversalCache,
        end_bound: usize,
        index: &Child,
    ) -> Vec<TraceState> {
        let subs = Self::completed_splits::<_, InnerNode>(trav, cache, index, end_bound)
            .into_iter()
            .filter_map(|(parent_offset, res)| res.ok().map(|locs| (parent_offset, locs)));
        self.leaves.filter_trace_states(trav, index, subs)
    }
    pub fn build(self) -> IntervalGraph {
        IntervalGraph {
            vertices: self.vertices,
            root_mode: self.root_mode,
            root: self.root,
            leaves: self.ctx.leaves,
        }
    }

    /// complete inner range offsets for non-roots
    pub fn augment_node(
        &mut self,
        ctx: NodeTraceContext,
    ) -> Vec<TraceState> {
        self.vertices
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
        self.vertices
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
        cache: &TraversalCache,
        index: &Child,
        end_bound: usize,
    ) -> N::CompleteSplitOutput {
        cache
            .entries
            .get(&index.vertex_index())
            .map(|e| SplitContext::new(e).complete_splits::<_, N>(trav, end_bound.into()))
            .unwrap_or_default()
    }
}
