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
    //cache: &'a SplitCache,
}
pub struct InnerPartition<'a> {
    pub left: OffsetSplitsRef<'a>,
    pub right: OffsetSplitsRef<'a>,
    //cache: &'a SplitCache,
}
pub struct LastPartition<'a> {
    pub inner: OffsetSplitsRef<'a>,
    //cache: &'a SplitCache,
}
pub trait PartitionRangeSplits: Sized {
    fn pattern_range_info(
        &self,
        pid: &PatternId,
        pattern: &Pattern,
        cache: &SplitCache,
    ) -> Result<(PatternPartitionInfo, Perfect), IndexedPartition>;

    /// bundle pattern range infos of each pattern
    fn info_bundle<'p>(
        self,
        ctx: &mut Partitioner<'p>,
    ) -> Result<PartitionBundle, IndexedPartition> {
        ctx.patterns().iter().map(|(pid, pattern)| {
            self.pattern_range_info(pid, pattern, ctx.cache)
        })
        .collect()
    }
    fn join_patterns<'p>(
        self,
        ctx: &mut Partitioner<'p>,
    ) -> IndexedPartition {
        match self.info_bundle(ctx) {
            Ok(bundle) => bundle.join(ctx),
            Err(part) => part,
        }
    }
    fn join<'p>(
        self,
        ctx: &mut Partitioner<'p>,
    ) -> IndexedPartition {
        match self.info_bundle(ctx) {
            Ok(bundle) => bundle.join(ctx),
            Err(part) => part,
        }
    }
}
impl<'a> PartitionRangeSplits for FirstPartition<'a> {
    fn pattern_range_info(
        &self,
        pid: &PatternId,
        pattern: &Pattern,
        cache: &SplitCache,
    ) -> Result<(PatternPartitionInfo, Perfect), IndexedPartition> {
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
                Err(IndexedPartition {
                    index: right_split.left,
                    perfect,
                })
            },
            (true, None) => {
                unreachable!("Invalid split position or invalid offset order");
            },
            (false, Some(right_split)) => Ok((
                PatternPartitionInfo {
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
            )),
            (false, None) => Ok((
                PatternPartitionInfo {
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
            )),

        }
    }
}
impl<'a> PartitionRangeSplits for InnerPartition<'a> {
    fn pattern_range_info(
        &self,
        pid: &PatternId,
        pattern: &Pattern,
        cache: &SplitCache,
    ) -> Result<(PatternPartitionInfo, Perfect), IndexedPartition> {
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
                Err(IndexedPartition {
                    index: right_split.left,
                    perfect,
                })
            },
            (true, None, Some(right_split)) => {
                // todo find inner partition
                unimplemented!();
                Err(IndexedPartition {
                    index: right_split.left,
                    perfect,
                })
            },
            (true, Some(left_split), None) => {
                unreachable!("Invalid split position or invalid offset order");
            },
            (true, None, None) =>
                Err(IndexedPartition {
                    index: right_child,
                    perfect,
                }),
            (false, Some(left_split), Some(right_split)) => Ok((
                PatternPartitionInfo {
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
            )),
            (false, None, Some(right_split)) => Ok((
                PatternPartitionInfo {
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
            )),
            (false, Some(left_split), None) => Ok((
                PatternPartitionInfo {
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
            )),
            (false, None, None) => Ok((
                PatternPartitionInfo {
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
            )),

        }
    }
}
impl<'a> PartitionRangeSplits for LastPartition<'a> {
    fn pattern_range_info(
        &self,
        pid: &PatternId,
        pattern: &Pattern,
        cache: &SplitCache,
    ) -> Result<(PatternPartitionInfo, Perfect), IndexedPartition> {
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
                Err(IndexedPartition {
                    index: left_split.right,
                    perfect,
                })
            },
            (true, None) => {
                //unreachable!("Invalid split position or invalid offset order");
                Err(IndexedPartition {
                    index: left_child,
                    perfect,
                })
            },
            (false, Some(left_split)) => Ok((
                PatternPartitionInfo {
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
            )),
            (false, None) => Ok((
                PatternPartitionInfo {
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
            )),

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