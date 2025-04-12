use super::position::{
    PosKey,
    SplitPositionCache,
};
use crate::{
    interval::partition::{
        Partition,
        ToPartition,
        info::{
            InfoPartition,
            range::{
                mode::Trace,
                role::{
                    In,
                    Post,
                    Pre,
                    RangeRole,
                },
                splits::{
                    OffsetIndexRange,
                    RangeOffsets,
                },
            },
        },
    },
    split::{
        position_splits,
        trace::SplitTraceState,
        vertex::output::RootMode,
    },
};
use context_search::trace::node::NodeTraceContext;
use derive_more::derive::{
    Deref,
    DerefMut,
};
use itertools::Itertools;
use std::{
    collections::BTreeMap,
    iter::FromIterator,
    num::NonZeroUsize,
};

#[derive(Debug, Default, Clone, PartialEq, Eq, Deref, DerefMut)]
pub struct SplitVertexCache
{
    pub positions: BTreeMap<NonZeroUsize, SplitPositionCache>,
}

impl SplitVertexCache
{
    pub fn new(
        pos: NonZeroUsize,
        entry: SplitPositionCache,
    ) -> Self
    {
        Self {
            positions: BTreeMap::from_iter([(pos, entry)]),
        }
    }
    pub fn augment_node(
        &mut self,
        ctx: NodeTraceContext,
    ) -> Vec<SplitTraceState>
    {
        let num_offsets = self.positions.len();
        let mut states = Vec::new();
        for len in 1..num_offsets
        {
            for start in 0..num_offsets - len + 1
            {
                let part = self
                    .offset_range_partition::<In<Trace>>(start..start + len);
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
    ) -> Vec<SplitTraceState>
    {
        let (splits, next) = match root_mode
        {
            RootMode::Infix =>
            {
                Self::add_inner_offsets(
                    ctx,
                    OffsetIndexRange::<In<Trace>>::get_splits(&(0..1), self),
                )
            }
            RootMode::Prefix =>
            {
                Self::add_inner_offsets::<Pre<Trace>, _>(
                    ctx,
                    OffsetIndexRange::<Pre<Trace>>::get_splits(&(0..0), self),
                )
            }
            RootMode::Postfix =>
            {
                Self::add_inner_offsets::<Post<Trace>, _>(
                    ctx,
                    OffsetIndexRange::<Post<Trace>>::get_splits(&(0..), self),
                )
            }
        };
        self.positions.extend(splits);
        next
    }
    pub fn pos_mut(
        &mut self,
        pos: NonZeroUsize,
    ) -> &mut SplitPositionCache
    {
        self.positions.entry(pos).or_default()
    }
    pub fn offset_range_partition<K: RangeRole>(
        &self,
        range: K::Range,
    ) -> Partition<K>
    {
        range.get_splits(self).to_partition()
    }
    pub fn inner_offsets<
        'a: 't,
        't,
        R: RangeRole<Mode = Trace>,
        P: ToPartition<R>,
    >(
        ctx: NodeTraceContext<'a>,
        part: P,
    ) -> Vec<NonZeroUsize>
    {
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
    pub fn add_inner_offsets<
        'a: 't,
        't,
        K: RangeRole<Mode = Trace>,
        P: ToPartition<K>,
    >(
        ctx: NodeTraceContext<'a>,
        part: P,
    ) -> (
        BTreeMap<NonZeroUsize, SplitPositionCache>,
        Vec<SplitTraceState>,
    )
//where K::Mode: ModeChildren::<K>,
    {
        let offsets = Self::inner_offsets(ctx, part);
        let splits: BTreeMap<_, _> = offsets
            .into_iter()
            .map(|offset| {
                (
                    offset,
                    SplitPositionCache::root(position_splits(
                        ctx.patterns,
                        offset,
                    )),
                )
            })
            .collect();
        let states = splits
            .iter()
            .flat_map(|(offset, cache)| {
                let key = PosKey::new(ctx.index, *offset);
                cache
                    .pattern_splits
                    .iter()
                    .flat_map(|(pid, pos)| {
                        pos.inner_offset.map(|inner_offset| {
                            let pattern = &ctx.patterns[pid];
                            SplitTraceState {
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
}
