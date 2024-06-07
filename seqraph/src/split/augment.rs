use std::{
    collections::BTreeMap,
    num::NonZeroUsize,
    sync::RwLockWriteGuard,
};

use itertools::Itertools;

use crate::{
    graph::Hypergraph,
    join::{
        context::node::context::NodeTraceContext,
        partition::{
            info::{
                range::{
                    role::{
                        In,
                        OffsetIndexRange,
                        Post,
                        Pre,
                        RangeRole,
                        Trace,
                    },
                    splits::RangeOffsets,
                },
                visit::VisitPartition,
            },
            AsPartition,
            Partition,
        },
    },
    split::{
        cache::{
            builder::SplitCacheBuilder,
            position::SplitPositionCache,
            TraceState,
        },
        position_splits,
    },
    traversal::{
        cache::key::SplitKey,
        folder::state::RootMode,
    },
    vertex::{
        child::Child,
        indexed::Indexed,
    },
};

use super::cache::vertex::SplitVertexCache;

impl SplitVertexCache {
    pub fn offset_range_partition<K: RangeRole>(
        &self,
        range: K::Range,
    ) -> Partition<K> {
        range.get_splits(self).as_partition()
    }
    pub fn inner_offsets<'a: 't, 't, K: RangeRole<Mode = Trace>, P: AsPartition<K>>(
        ctx: NodeTraceContext<'a>,
        part: P,
    ) -> Vec<NonZeroUsize> {
        part.info_partition(&ctx)
            .map(|bundle|
                //let merges = range_map.range_sub_merges(start..start + len);
                bundle.patterns.into_iter().flat_map(|(_pid, info)|
                    info.inner_range.map(|range| {
                        let splits = range.offsets.as_splits(ctx);
                        Self::inner_offsets(
                            ctx,
                            splits,
                        )
                    })
                )
                    .flatten()
                    .collect())
            .unwrap_or_default()
    }
    pub fn add_inner_offsets<'a: 't, 't, K: RangeRole<Mode = Trace>, P: AsPartition<K>>(
        ctx: NodeTraceContext<'a>,
        part: P,
    ) -> (BTreeMap<NonZeroUsize, SplitPositionCache>, Vec<TraceState>)
//where K::Mode: ModeChildren::<K>,
    {
        let offsets = Self::inner_offsets(ctx, part);
        let splits: BTreeMap<_, _> = offsets
            .into_iter()
            .map(|offset| {
                (
                    offset,
                    SplitPositionCache::root(position_splits(ctx.patterns, offset)),
                )
            })
            .collect();
        let states = splits
            .iter()
            .flat_map(|(offset, cache)| {
                let key = SplitKey::new(ctx.index, *offset);
                cache
                    .pattern_splits
                    .iter()
                    .flat_map(|(pid, pos)| {
                        pos.inner_offset.map(|inner_offset| {
                            let pattern = &ctx.patterns[pid];
                            TraceState {
                                index: pattern[pos.sub_index],
                                offset: inner_offset,
                                prev: key,
                            }
                        })
                    })
                    .collect_vec()
            })
            .collect();
        (splits, states)
    }
    pub fn augment_node(
        &mut self,
        ctx: NodeTraceContext,
    ) -> Vec<TraceState> {
        let num_offsets = self.positions.len();
        let mut states = Vec::new();
        for len in 1..num_offsets {
            for start in 0..num_offsets - len + 1 {
                let part = self.offset_range_partition::<In<Trace>>(start..start + len);
                let (splits, next) = Self::add_inner_offsets(ctx, part);
                self.positions.extend(splits);
                states.extend(next);
            }
        }
        states
    }
    pub fn augment_root(
        &mut self,
        ctx: NodeTraceContext,
        root_mode: RootMode,
    ) -> Vec<TraceState> {
        let (splits, next) = match root_mode {
            RootMode::Infix => Self::add_inner_offsets(
                ctx,
                OffsetIndexRange::<In<Trace>>::get_splits(&(0..1), self),
            ),
            RootMode::Prefix => Self::add_inner_offsets::<Pre<Trace>, _>(
                ctx,
                OffsetIndexRange::<Pre<Trace>>::get_splits(&(0..0), self),
            ),
            RootMode::Postfix => Self::add_inner_offsets::<Post<Trace>, _>(
                ctx,
                OffsetIndexRange::<Post<Trace>>::get_splits(&(0..), self),
            ),
        };
        self.positions.extend(splits);
        next
    }
}

impl SplitCacheBuilder {
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
    pub fn augment_nodes<I: IntoIterator<Item = Child>>(
        &mut self,
        graph: &RwLockWriteGuard<'_, Hypergraph>,
        nodes: I,
    ) {
        for c in nodes {
            let new = self.augment_node(NodeTraceContext::new(graph, c));
            // todo: force order
            self.states.extend(new.into_iter());
        }
    }
}
