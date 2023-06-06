use crate::*;

#[derive(Debug, Default, Clone)]
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
    pub fn offset_range_partition<'a, K: RangeRole>(&'a self, range: K::Range) -> PartitionRef<'a, K> {
        range.get_partition(self)
    }
    pub fn inner_offsets<'a, K: RangeRole, P: AsPartition<'a, K>>(
        ctx: BundlingContext,
        part: P,
    ) -> Vec<NonZeroUsize> {
        part.info_bundle(&ctx).map(|bundle|
            //let merges = range_map.range_sub_merges(start..start + len);
            bundle.patterns.into_iter().flat_map(|(pid, info)|
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
        'a,
        C: AsBundlingContext<'a>,
        K: RangeRole,
        P: AsPartition<'a, K>,
    >(
        &mut self,
        ctx: C,
        part: P,
    ) -> Vec<TraceState> {
        let ctx = ctx.as_bundling_context();
        let offsets = Self::inner_offsets(ctx, part);
        let splits = offsets.into_iter().map(|offset|
            (offset, SplitPositionCache::root(
                position_splits(ctx.patterns, offset)
            ))
        );
        let states = splits.into_iter().flat_map(|(offset, cache)| {
            let key = SplitKey::new(ctx.index, offset);
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
        })
        .collect();
        self.positions.extend(splits);
        states
    }
    pub fn complete_node<'a>(
        &mut self,
        ctx: JoinContext<'a>,
    ) -> Vec<TraceState> {
        let num_offsets = self.positions.len();
        let mut states = Vec::new();
        let ctx = ctx.as_bundling_context();
        for len in 1..num_offsets {
            for start in 0..num_offsets-len+1 {
                let part = self.offset_range_partition::<In>(start..start + len);
                states.extend(
                    self.add_inner_offsets(
                        ctx,
                        part,
                    )
                );
            } 
        }
        states
    }
    pub fn complete_root<'a>(
        &mut self,
        ctx: JoinContext<'a>,
        root_mode: RootMode
    ) -> Vec<TraceState> {
        match root_mode {
            RootMode::Infix =>
                self.add_inner_offsets(
                    ctx,
                    OffsetIndexRange::<In>::get_partition(&(0..1), self),
                ),
            RootMode::Prefix =>
                self.add_inner_offsets(
                    ctx,
                    OffsetIndexRange::<Pre>::get_partition(&(0..1), self),
                ),
            RootMode::Postfix =>
                self.add_inner_offsets(
                    ctx,
                    (0..).get_partition(self),
                ),
        }
    }
}
#[derive(Debug, Clone)]
pub struct PatternSplitPos {
    pub inner_offset: Option<NonZeroUsize>,
    pub sub_index: usize,
}
pub type PatternSubSplits = HashMap<PatternId, PatternSplitPos>;
pub trait ToPatternSubSplits {
    fn to_pattern_sub_splits(self) -> PatternSubSplits;
}
impl ToPatternSubSplits for PatternSubSplits {
    fn to_pattern_sub_splits(self) -> PatternSubSplits {
        self
    }
}
impl ToPatternSubSplits for Vec<SubSplitLocation> {
    fn to_pattern_sub_splits(self) -> PatternSubSplits {
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
}impl ToPatternSubSplits for OffsetSplits {
    fn to_pattern_sub_splits(self) -> PatternSubSplits {
        self.splits
    }
}
#[derive(Debug, Default, Clone)]
pub struct SplitPositionCache {
    pub top: HashSet<SplitKey>,
    pub pattern_splits: PatternSubSplits,
    pub final_split: Option<FinalSplit>,
}

impl std::ops::Sub<PatternSubDeltas> for SplitPositionCache {
    type Output = Self;
    fn sub(mut self, rhs: PatternSubDeltas) -> Self::Output {
        self.pattern_splits.iter_mut()
            .for_each(|(pid, pos)|
                pos.sub_index -= rhs[pid]
            );
        self
    }
}
impl Borrow<PatternSubSplits> for SplitPositionCache {
    fn borrow(&self) -> &PatternSubSplits {
        &self.pattern_splits
    }
}
impl BorrowMut<PatternSubSplits> for SplitPositionCache {
    fn borrow_mut(&mut self) -> &mut PatternSubSplits {
        &mut self.pattern_splits
    }
}
impl From<SubSplitLocation> for (PatternId, PatternSplitPos) {
    fn from(sub: SubSplitLocation) -> Self {
        (
            sub.location.pattern_id,
            PatternSplitPos {
                inner_offset: sub.inner_offset,
                sub_index: sub.location.sub_index,
            }
        )
    }
}
impl SplitPositionCache {
    pub fn root(subs: impl ToPatternSubSplits) -> Self {
        Self {
            top: HashSet::default(),
            pattern_splits: subs.to_pattern_sub_splits(),
            final_split: None,
        }
    }
    pub fn new(prev: SplitKey, subs: Vec<SubSplitLocation>) -> Self {
        Self {
            top: HashSet::from_iter(Some(prev)),
            pattern_splits: subs.into_iter().map(Into::into).collect(),
            final_split: None,
        }
    }
    pub fn find_clean_split(&self) -> Option<SubLocation> {
        self.pattern_splits.iter().find_map(|(pid, s)|
            s.inner_offset.is_none().then_some(
                SubLocation {
                    pattern_id: *pid,
                    sub_index: s.sub_index,
                }
            )
        )
    }
    //pub fn add_location_split(&mut self, location: SubLocation, split: Split) {
    //    self.pattern_splits.insert(location, split);
    //}
    //pub fn join_splits(&mut self, indexer: &mut Indexer, key: &SplitKey) -> Split {
    //    let (l, r): (Vec<_>, Vec<_>) = self.pattern_splits
    //        .drain()
    //        .map(|(_, s)| (s.left, s.right))
    //        .unzip();
    //    // todo detect existing splits
    //    let mut graph = indexer.graph_mut();
    //    let lc = graph.insert_patterns(l);
    //    let rc = graph.insert_patterns(r);
    //    graph.add_pattern_with_update(&key.index, vec![lc, rc]);
    //    let split = Split {
    //        left: vec![lc],
    //        right: vec![rc],
    //    };
    //    self.final_split = Some(split.clone());
    //    split
    //}
}
//impl From<Split> for SplitPositionCache {
//    fn from(split: Split) -> Self {
//        Self {
//            pattern_splits: Default::default(),
//            final_split: Some(split),
//        }
//    }
//}
