use crate::*;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct SplitVertexCache {
    pub positions: BTreeMap<NonZeroUsize, SplitPositionCache>,
}
impl SplitVertexCache {
    pub fn new(pos: NonZeroUsize, entry: SplitPositionCache) -> Self {
        Self {
            positions: BTreeMap::from_iter([
                (
                    pos,
                    entry,
                )
            ]),
        }
    }
    pub fn pos_mut(&mut self, pos: NonZeroUsize) -> &mut SplitPositionCache {
        self.positions.entry(pos)
            .or_default()
    }
    pub fn offset_range_partition<'a, K: RangeRole>(&'a self, range: K::Range) -> Partition<K> {
        range.get_splits(self).as_partition()
    }
    pub fn inner_offsets<'a: 't, 't, K: RangeRole<Mode = Trace>, P: AsPartition<K>>(
        ctx: TraceContext<'a>,
        part: P,
    ) -> Vec<NonZeroUsize> {
        part.visit_partition(&ctx).map(|bundle|
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
            .collect()
        ).unwrap_or_default()
    }
    pub fn add_inner_offsets<
        'a: 't,
        't,
        K: RangeRole<Mode = Trace>,
        P: AsPartition<K>,
    >(
        ctx: TraceContext<'a>,
        part: P,
    ) -> (BTreeMap<NonZeroUsize, SplitPositionCache>, Vec<TraceState>)
        where K::Mode: ModeChildren::<K>,
    {
        let offsets = Self::inner_offsets(ctx, part);
        let splits: BTreeMap<_, _> = offsets.into_iter().map(|offset|
            (offset, SplitPositionCache::root(
                position_splits(ctx.patterns, offset)
            ))
        ).collect();
        let states = splits.iter().flat_map(|(offset, cache)| {
            let key = SplitKey::new(ctx.index, *offset);
            cache.pattern_splits.iter().flat_map(|(pid, pos)|
                pos.inner_offset.map(|inner_offset| {
                    let pattern = &ctx.patterns[pid];
                    TraceState {
                        index: pattern[pos.sub_index],
                        offset: inner_offset,
                        prev: key,
                    }
                })
            )
            .collect_vec()
        })
        .collect();
        (splits, states)
    }
    pub fn complete_node<'a>(
        &mut self,
        ctx: TraceContext<'a>,
    ) -> Vec<TraceState> {
        let num_offsets = self.positions.len();
        let mut states = Vec::new();
        let ctx = ctx.as_trace_context();
        for len in 1..num_offsets {
            for start in 0..num_offsets-len+1 {
                let part = self.offset_range_partition::<In<Trace>>(start..start + len);
                let (splits, next) = Self::add_inner_offsets(
                    ctx,
                    part,
                );
                self.positions.extend(splits);
                states.extend(next);
            } 
        }
        states
    }
    pub fn complete_root<'a>(
        &mut self,
        ctx: TraceContext<'a>,
        root_mode: RootMode
    ) -> Vec<TraceState> {
        let (splits, next) = match root_mode {
            RootMode::Infix =>
                Self::add_inner_offsets(
                    ctx,
                    OffsetIndexRange::<In<Trace>>::get_splits(&(0..1), self),
                ),
            RootMode::Prefix =>
                Self::add_inner_offsets::<Pre<Trace>, _>(
                    ctx,
                    OffsetIndexRange::<Pre<Trace>>::get_splits(&(0..0), self),
                ),
            RootMode::Postfix =>
                Self::add_inner_offsets::<Post<Trace>, _>(
                    ctx,
                    OffsetIndexRange::<Post<Trace>>::get_splits(&(0..), self),
                ),
        };
        self.positions.extend(splits);
        next
    }
}
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct PatternSplitPos {
    pub inner_offset: Option<NonZeroUsize>,
    pub sub_index: usize,
}
pub type VertexSplitPos = HashMap<PatternId, PatternSplitPos>;
pub trait ToVertexSplitPos {
    fn to_vertex_split_pos(self) -> VertexSplitPos;
}
impl ToVertexSplitPos for VertexSplitPos {
    fn to_vertex_split_pos(self) -> VertexSplitPos {
        self
    }
}
impl ToVertexSplitPos for Vec<SubSplitLocation> {
    fn to_vertex_split_pos(self) -> VertexSplitPos {
        self.into_iter().map(|loc|
            (
                loc.location.pattern_id,
                PatternSplitPos {
                    inner_offset: loc.inner_offset,
                    sub_index: loc.location.sub_index,
                },
            )
        )
        .collect()
    }
}impl ToVertexSplitPos for OffsetSplits {
    fn to_vertex_split_pos(self) -> VertexSplitPos {
        self.splits
    }
}