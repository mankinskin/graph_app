use itertools::Itertools;
use std::{
    collections::BTreeMap,
    iter::FromIterator,
    num::NonZeroUsize,
};

use crate::{
    partition::{
        context::NodeTraceContext,
        info::{
            InfoPartition,
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
        },
        Partition,
        ToPartition,
    },
    split::cache::position::SplitPositionCache,
    traversal::cache::{
        entry::RootMode,
        key::SplitKey,
    },
};

use super::{
    position_splits,
    TraceState,
};

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct SplitVertexCache {
    pub positions: BTreeMap<NonZeroUsize, SplitPositionCache>,
}

impl SplitVertexCache {
    pub fn new(
        pos: NonZeroUsize,
        entry: SplitPositionCache,
    ) -> Self {
        Self {
            positions: BTreeMap::from_iter([(pos, entry)]),
        }
    }
    pub fn pos_mut(
        &mut self,
        pos: NonZeroUsize,
    ) -> &mut SplitPositionCache {
        self.positions.entry(pos).or_default()
    }
    pub fn offset_range_partition<K: RangeRole>(
        &self,
        range: K::Range,
    ) -> Partition<K> {
        range.get_splits(self).to_partition()
    }
    pub fn inner_offsets<'a: 't, 't, R: RangeRole<Mode = Trace>, P: ToPartition<R>>(
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
    pub fn add_inner_offsets<'a: 't, 't, K: RangeRole<Mode = Trace>, P: ToPartition<K>>(
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
