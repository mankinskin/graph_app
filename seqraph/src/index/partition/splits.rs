use crate::*;

type OptCleanSplit = Option<SubLocation>;
//pub enum RootPartitions {
//    Prefix(Child, OptCleanSplit, Pattern),
//    Postfix(Pattern, OptCleanSplit, Child),
//    Infix(Pattern, OptCleanSplit, Child, OptCleanSplit, Pattern),
//}
//impl RootPartitions {
//    pub fn inner(&self) -> &Child {
//        match self {
//            Self::Infix(_, _, inner, _, _) => inner,
//            Self::Prefix(inner, _, _) => inner,
//            Self::Postfix(_, _, inner) => inner,
//        }
//    }
//}

pub struct FirstPartition<'a> {
    pub inner: OffsetSplitsRef<'a>,
}
pub struct InnerPartition<'a> {
    pub left: OffsetSplitsRef<'a>,
    pub right: OffsetSplitsRef<'a>,
}
pub struct LastPartition<'a> {
    pub inner: OffsetSplitsRef<'a>,
}
#[derive(Debug)]
pub struct OffsetSplits {
    pub offset: NonZeroUsize,
    pub splits: PatternSubSplits,
}
#[derive(Debug, Clone, Copy)]
pub struct OffsetSplitsRef<'a> {
    pub offset: NonZeroUsize,
    pub splits: &'a PatternSubSplits,
}
pub trait AsOffsetSplits<'a>: 'a {
    fn as_offset_splits<'t>(&'t self) -> OffsetSplitsRef<'t> where 'a: 't;
}
impl<'a, O: AsOffsetSplits<'a>> AsOffsetSplits<'a> for &'a O {
    fn as_offset_splits<'t>(&'t self) -> OffsetSplitsRef<'t> where 'a: 't {
        (*self).as_offset_splits()
    }
}
impl<'a> AsOffsetSplits<'a> for OffsetSplits {
    fn as_offset_splits<'t>(&'t self) -> OffsetSplitsRef<'t> where 'a: 't {
        OffsetSplitsRef {
            offset: self.offset,
            splits: &self.splits,
        }
    }
}
impl<'a, N: Borrow<NonZeroUsize> + 'a> AsOffsetSplits<'a> for (N, &'a SplitPositionCache) {
    fn as_offset_splits<'t>(&'t self) -> OffsetSplitsRef<'t> where 'a: 't {
        OffsetSplitsRef {
            offset: self.0.borrow().clone(),
            splits: &self.1.pattern_splits,
        }
    }
}
impl<'a, N: Borrow<NonZeroUsize> + 'a> AsOffsetSplits<'a> for (N, SplitPositionCache) {
    fn as_offset_splits<'t>(&'t self) -> OffsetSplitsRef<'t> where 'a: 't {
        OffsetSplitsRef {
            offset: self.0.borrow().clone(),
            splits: &self.1.pattern_splits,
        }
    }
}
impl<'a> AsOffsetSplits<'a> for OffsetSplitsRef<'a> {
    fn as_offset_splits<'t>(&'t self) -> OffsetSplitsRef<'t> where 'a: 't {
        *self
    }
}

pub trait HasSubSplits {
    type Split: Borrow<PatternSubSplits>;
    fn sub_splits<'s>(&self, cache: &'s SplitCache) -> &'s BTreeMap<NonZeroUsize, Self::Split>;
}
impl<C: Indexed> HasSubSplits for C {
    type Split = SplitPositionCache;
    fn sub_splits<'s>(&self, cache: &'s SplitCache) -> &'s BTreeMap<NonZeroUsize, Self::Split> {
        &cache.entries.get(&self.index()).unwrap().positions
    }
}
pub trait AsPartition<'a>: 'a {
    type Partition<'t>: PartitionRangeSplits where 'a: 't;
    fn as_partition<'t>(&'t self) -> Self::Partition<'t> where 'a: 't;
}
impl<'a, A: AsOffsetSplits<'a>, B: AsOffsetSplits<'a>> AsPartition<'a> for (A, B) {
    type Partition<'t> = InnerPartition<'t> where 'a: 't;
    fn as_partition<'t>(&'t self) -> Self::Partition<'t> where 'a: 't {
        InnerPartition {
            left: self.0.as_offset_splits(),
            right: self.1.as_offset_splits(),
        }
    }
}
impl<'a, B: AsOffsetSplits<'a>> AsPartition<'a> for ((), B) {
    type Partition<'t> = FirstPartition<'t> where 'a: 't;
    fn as_partition<'t>(&'t self) -> Self::Partition<'t> where 'a: 't {
        FirstPartition {
            inner: self.1.as_offset_splits(),
        }
    }
}
impl<'a, A: AsOffsetSplits<'a>> AsPartition<'a> for (A, ()) {
    type Partition<'t> = LastPartition<'t> where 'a: 't;
    fn as_partition<'t>(&'t self) -> Self::Partition<'t> where 'a: 't {
        LastPartition {
            inner: self.0.as_offset_splits(),
        }
    }
}
#[derive(Debug)]
pub struct PatternPartitionResult {
    pub info: PatternPartitionInfo,
    pub perfect: Perfect,
    pub delta: usize,
}
pub trait PartitionRangeSplits: Sized {
    fn pattern_partition(
        &self,
        pid: &PatternId,
        pattern: &Pattern,
        cache: &SplitCache,
    ) -> Result<PatternPartitionResult, Child>;

    /// bundle pattern range infos of each pattern
    fn info_bundle<'p>(
        self,
        ctx: &mut Partitioner<'p>,
    ) -> Result<PartitionBundle, Child> {
        ctx.patterns().iter().map(|(pid, pattern)| {
            self.pattern_partition(pid, pattern, ctx.cache)
        })
        .collect()
    }
    fn join<'p>(
        self,
        ctx: &mut Partitioner<'p>,
    ) -> Result<JoinedPartition, Child> {
        match self.info_bundle(ctx) {
            Ok(bundle) => Ok(bundle.join(ctx)),
            Err(part) => Err(part),
        }
    }
}
impl<'a, P: AsPartition<'a>> PartitionRangeSplits for P {
    fn pattern_partition(
        &self,
        pid: &PatternId,
        pattern: &Pattern,
        cache: &SplitCache,
    ) -> Result<PatternPartitionResult, Child> {
        self.as_partition().pattern_partition(
            pid,
            pattern,
            cache,
        )
    }
}
impl<'a> PartitionRangeSplits for FirstPartition<'a> {
    fn pattern_partition(
        &self,
        pid: &PatternId,
        pattern: &Pattern,
        cache: &SplitCache,
    ) -> Result<PatternPartitionResult, Child> {
        // todo detect if prev offset is in same index (to use inner partition as result)
        let right_pos = self.inner.splits.get(pid).unwrap();
        let urange = (0, self.inner.offset.get());

        let left_child = pattern[0];

        let right_child = pattern[right_pos.sub_index];
        let right_split = right_pos.inner_offset.map(|inner_offset|
            cache.expect_final_split(&SplitKey::new(right_child, inner_offset))
        );

        // get split parts in this partition
        let context_range = 1..right_pos.sub_index;
        let context = pattern.get(context_range.clone()).unwrap_or(&[]);
        let context_range = (!context.is_empty()).then(|| context_range);

        let perfect = (
            None,
            right_split.is_none().then_some(*pid),
        );
        let delta = right_pos.sub_index.saturating_sub(2);

        match (
            0 == right_pos.sub_index,
            right_split,
        ) {
            (true, Some(right_split)) => {
                // todo find inner partition
                unimplemented!();
                //self.get_partition(
                //    merges,
                //    offsets,
                //    range,
                //)
                Err(right_split.left)
            },
            (true, None) => {
                unreachable!("Invalid split position or invalid offset order");
            },
            (false, Some(right_split)) => Ok(PatternPartitionResult {
                info: PatternPartitionInfo {
                    pattern_id: *pid,
                    left: left_child,
                    right: right_split.left,
                    inner_range: context_range.map(|range|
                        InnerRangeInfo {
                            range,
                            offsets: to_non_zero_range(
                                urange.0 + left_child.width(),
                                urange.1 - right_split.left.width()
                            ),
                        }
                    ),
                },
                perfect,
                delta,
            }),
            (false, None) => Ok(PatternPartitionResult {
                info: PatternPartitionInfo {
                    pattern_id: *pid,
                    left: left_child,
                    right: right_child,
                    inner_range: context_range.map(|range|
                        InnerRangeInfo {
                            range,
                            offsets: to_non_zero_range(
                                urange.0 + left_child.width(),
                                urange.1 - right_child.width()
                            ),
                        }
                    ),
                },
                perfect,
                delta,
            }),

        }
    }
}
impl<'a> PartitionRangeSplits for InnerPartition<'a> {
    fn pattern_partition(
        &self,
        pid: &PatternId,
        pattern: &Pattern,
        cache: &SplitCache,
    ) -> Result<PatternPartitionResult, Child> {
        // todo detect if prev offset is in same index (to use inner partition as result)
        let (left_pos, right_pos) = (self.left.splits.get(pid).unwrap(), self.right.splits.get(pid).unwrap());
        let urange = (self.left.offset.get(), self.right.offset.get());

        let left_child = pattern[left_pos.sub_index];
        let left_split = left_pos.inner_offset.map(|inner_offset|
            cache.expect_final_split(&SplitKey::new(left_child, inner_offset))
        );

        let right_child = pattern[right_pos.sub_index];
        let right_split = right_pos.inner_offset.map(|inner_offset|
            cache.expect_final_split(&SplitKey::new(right_child, inner_offset))
        );

        // get split parts in this partition
        let context_range = left_pos.sub_index+1..right_pos.sub_index;
        let context = pattern.get(context_range.clone()).unwrap_or(&[]);
        let context_range = (!context.is_empty()).then(|| context_range);

        let perfect = (
            left_split.is_none().then_some(*pid),
            right_split.is_none().then_some(*pid),
        );
        let delta = right_pos.sub_index.saturating_sub(left_pos.sub_index+2);

        match (
            left_pos.sub_index == right_pos.sub_index,
            left_split,
            right_split,
        ) {
            (true, Some(left_split), Some(right_split)) => {
                // todo find inner partition
                unimplemented!();
                //self.get_partition(
                //    merges,
                //    offsets,
                //    range,
                //)
                Err(right_split.left)
            },
            (true, None, Some(right_split)) => {
                // todo find inner partition
                unimplemented!();
                Err(right_split.left)
            },
            (true, Some(left_split), None) => {
                unreachable!("Invalid split position or invalid offset order");
            },
            (true, None, None) => Err(right_child),
            (false, Some(left_split), Some(right_split)) => Ok(PatternPartitionResult {
                info: PatternPartitionInfo {
                    pattern_id: *pid,
                    left: left_split.right,
                    right: right_split.left,
                    inner_range: context_range.map(|range|
                        InnerRangeInfo {
                            range,
                            offsets: to_non_zero_range(
                                urange.0 + left_split.right.width(),
                                urange.1 - right_split.left.width()
                            ),
                        }
                    ),
                },
                perfect,
                delta,
            }),
            (false, None, Some(right_split)) => Ok(PatternPartitionResult {
                info: PatternPartitionInfo {
                    pattern_id: *pid,
                    left: left_child,
                    right: right_split.left,
                    inner_range: context_range.map(|range|
                        InnerRangeInfo {
                            range,
                            offsets: to_non_zero_range(
                                urange.0 + left_child.width(),
                                urange.1 - right_split.left.width()
                            ),
                        }
                    ),
                },
                perfect,
                delta,
        }),
            (false, Some(left_split), None) => Ok(PatternPartitionResult {
                info: PatternPartitionInfo {
                    pattern_id: *pid,
                    left: left_split.right,
                    right: right_child,
                    inner_range: context_range.map(|range|
                        InnerRangeInfo {
                            range,
                            offsets: to_non_zero_range(
                                urange.0 + left_split.right.width(),
                                urange.1 - right_child.width()
                            ),
                        }
                    ),
                },
                perfect,
                delta,
            }),
            (false, None, None) => Ok(PatternPartitionResult {
                info: PatternPartitionInfo {
                    pattern_id: *pid,
                    left: left_child,
                    right: right_child,
                    inner_range: context_range.map(|range|
                        InnerRangeInfo {
                            range,
                            offsets: to_non_zero_range(
                                urange.0 + left_child.width(),
                                urange.1 - right_child.width()
                            ),
                        }
                    ),
                },
                perfect,
                delta,
            }),

        }
    }
}
impl<'a> PartitionRangeSplits for LastPartition<'a> {
    fn pattern_partition(
        &self,
        pid: &PatternId,
        pattern: &Pattern,
        cache: &SplitCache,
    ) -> Result<PatternPartitionResult, Child> {
        // todo detect if prev offset is in same index (to use inner partition as result)
        let left_pos = self.inner.splits.get(pid).unwrap();
        let urange = (0, self.inner.offset.get());

        let left_child = pattern[left_pos.sub_index];
        let left_split = left_pos.inner_offset.map(|inner_offset|
            cache.expect_final_split(&SplitKey::new(left_child, inner_offset))
        );

        let len = pattern.len();
        let right_child = pattern[len - 1];

        // get split parts in this partition
        let context_range = left_pos.sub_index..len-1;
        let context = pattern.get(context_range.clone()).unwrap_or(&[]);
        let context_range = (!context.is_empty()).then(|| context_range);

        let perfect = (
            left_split.is_none().then_some(*pid),
            None,
        );
        let delta = len.saturating_sub(left_pos.sub_index+1);

        match (
            left_pos.sub_index == len - 1,
            left_split,
        ) {
            (true, Some(left_split)) => {
                // todo find inner partition
                unimplemented!();
                //self.get_partition(
                //    merges,
                //    offsets,
                //    range,
                //)
                Err(left_split.right)
            },
            (true, None) => {
                //unreachable!("Invalid split position or invalid offset order");
                Err(left_child)
            },
            (false, Some(left_split)) => Ok(PatternPartitionResult {
                info: PatternPartitionInfo {
                    pattern_id: *pid,
                    left: left_split.right,
                    right: right_child,
                    inner_range: context_range.map(|range|
                        InnerRangeInfo {
                            range,
                            offsets: to_non_zero_range(
                                urange.0 + left_split.right.width(),
                                urange.1 - right_child.width()
                            ),
                        }
                    ),
                },
                perfect,
                delta,
            }),
            (false, None) => Ok(PatternPartitionResult {
                info: PatternPartitionInfo {
                    pattern_id: *pid,
                    left: left_child,
                    right: right_child,
                    inner_range: context_range.map(|range|
                        InnerRangeInfo {
                            range,
                            offsets: to_non_zero_range(
                                urange.0 + left_child.width(),
                                urange.1 - right_child.width()
                            ),
                        }
                    ),
                },
                perfect,
                delta,
            }),

        }
    }
}
fn to_non_zero_range(
    l: usize,
    r: usize,
) -> (NonZeroUsize, NonZeroUsize) {
    (
        NonZeroUsize::new(l).unwrap(),
        NonZeroUsize::new(r).unwrap(),
    )
}