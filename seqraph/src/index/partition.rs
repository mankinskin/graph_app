use crate::*;
pub enum RootPartitions {
    Prefix(Child, OptCleanSplit, Pattern),
    Postfix(Pattern, OptCleanSplit, Child),
    Infix(Pattern, OptCleanSplit, Child, OptCleanSplit, Pattern),
}
type OptCleanSplit = Option<SubLocation>;
impl RootPartitions {
    pub fn inner(&self) -> &Child {
        match self {
            Self::Infix(_, _, inner, _, _) => inner,
            Self::Prefix(inner, _, _) => inner,
            Self::Postfix(_, _, inner) => inner,
        }
    }
}
impl Indexer {
    pub fn first_partition(
        &mut self,
        cache: &SplitCache,
        pos_cache: &SplitPositionCache,
        index: &Child,
    ) -> Result<Vec<(PatternId, Range<usize>, Option<Child>)>, Child> {
        let graph = self.graph_mut();
        pos_cache.pattern_splits.iter().map(|(&pid, pos)| {
            // todo detect if prev offset is in same index (to use inner partition as result)
            let pattern = graph.expect_pattern_at(&index.to_pattern_location(pid));

            let child = pattern[pos.sub_index];
            // get split parts in this partition
            let context = &pattern[0..pos.sub_index];
            let inner_split = pos.inner_offset.map(|inner_offset|
                cache.expect_final_split(&SplitKey::new(child, inner_offset))
            );
            match ((!context.is_empty()).then_some(context), inner_split) {
                // finish with inner split
                (None, Some(inner_split)) => {
                    Err(inner_split.left)
                },
                (Some(_), Some(inner_split)) => {
                    Ok((pid, 0..pos.sub_index, Some(inner_split.left)))
                },
                (Some(context), None) => {
                    if context.len() == 1 {
                        Err(context[0])
                    } else {
                        Ok((pid, 0..pos.sub_index, None))
                    }
                },
                (None, None) => unreachable!(), // split at 0
            }
        }).collect()
    }
    pub fn last_partition(
        &mut self,
        cache: &SplitCache,
        prev_cache: &SplitPositionCache,
        index: &Child,
    ) -> Result<Vec<(PatternId, Option<Child>, RangeFrom<usize>)>, Child> {
        let graph = self.graph_mut();
        prev_cache.pattern_splits.iter().map(|(&pid, pos)| {
            let pattern = graph.expect_pattern_at(&index.to_pattern_location(pid));

            let child = pattern[pos.sub_index];
            let context = pattern.get(pos.sub_index+1..).unwrap_or(&[]);
            let inner_split = pos.inner_offset.map(|inner_offset|
                cache.expect_final_split(&SplitKey::new(child, inner_offset))
            );
            match ((!context.is_empty()).then_some(context), inner_split) {
                // finish with inner split
                (None, Some(inner_split)) => {
                    Err(inner_split.right)
                },
                (Some(_), Some(inner_split)) => {
                    Ok((pid, Some(inner_split.right), pos.sub_index + 1..))
                },
                (Some(_), None) => {
                    if context.len() == 1 {
                        Err(context[0])
                    } else {
                        Ok((pid, None, pos.sub_index..))
                    }
                },
                (None, None) => Err(child),
            }
        }).collect()
    }
    pub fn inner_partition(
        &mut self,
        cache: &SplitCache,
        prev_cache: &SplitPositionCache,
        pos_cache: &SplitPositionCache,
        index: &Child,
    ) -> Child {
        let mut graph = self.graph_mut();
        match pos_cache.pattern_splits.iter().map(|(&pid, pos)| {
            // todo detect if prev offset is in same index (to use inner partition as result)
            let prev_pos = prev_cache.pattern_splits.get(&pid).unwrap();

            let pattern = graph.expect_pattern_at(&index.to_pattern_location(pid));
            let prev_child = pattern[prev_pos.sub_index];
            let prev_split = prev_pos.inner_offset.map(|inner_offset|
                cache.expect_final_split(&SplitKey::new(prev_child, inner_offset))
            );
            let child = pattern[pos.sub_index];
            let inner_split = pos.inner_offset.map(|inner_offset|
                cache.expect_final_split(&SplitKey::new(child, inner_offset))
            );
            // get split parts in this partition
            let context_range = prev_pos.sub_index+1..pos.sub_index;
            let context = pattern.get(prev_pos.sub_index+1..pos.sub_index).unwrap_or(&[]);
            let context_range = (!context.is_empty()).then(|| context_range);
            match (
                prev_pos.sub_index == pos.sub_index,
                prev_split,
                inner_split,
            ) {
                (true, Some(prev_split), Some(inner_split)) => {
                    // todo find inner partition
                    unimplemented!();
                    //self.get_partition(
                    //    merges,
                    //    offsets,
                    //    range,
                    //)
                    Err(inner_split.left)
                },
                (true, None, Some(inner_split)) => {
                    // todo find inner partition
                    unimplemented!();
                    Err(inner_split.left)
                },
                (true, Some(prev_split), None) => {
                    unreachable!("Invalid split position or invalid offset order");
                },

                (true, None, None) => Err(child),

                (false, Some(prev_split), Some(inner_split)) => {
                    Ok((pid, prev_split.right, context_range, inner_split.left))
                },
                (false, None, Some(inner_split)) => {
                    Ok((pid, prev_child, context_range, inner_split.left))
                },
                (false, Some(prev_split), None) => {
                    Ok((pid, prev_split.right, context_range, child))
                },
                (false, None, None) => {
                    Ok((pid, prev_child, context_range, child))
                },

            }
        }).collect::<Result<Vec<_>, Child>>() {
            Ok(bundle) => {
                let bundle = bundle.into_iter()
                    .map(|(pid, left, context_range, right)| {
                        if let Some(context) = context_range.map(|r|
                            graph.insert_range_in(
                                index.to_pattern_location(pid),
                                r,
                            ).unwrap()
                        ) {
                            vec![left, context, right]
                        } else {
                            vec![left, right]
                        }
                    })
                    .collect_vec();
                graph.insert_patterns(
                    bundle
                )
            },
            Err(child) => child,
        }
    }
    pub fn first_child_partition(
        &mut self,
        cache: &SplitCache,
        pos_cache: &SplitPositionCache,
        index: &Child,
    ) -> Child {
        match self.first_partition(
            cache,
            pos_cache,
            index,
        ) {
            Ok(bundle) => {
                let mut graph = self.graph_mut();
                if bundle.len() > 1 {
                    let bundle = bundle.into_iter()
                        .map(|(pid, context_range, inner)| {
                            let context = graph.insert_range_in(
                                index.to_pattern_location(pid),
                                context_range,
                            ).unwrap();
                            vec![context, inner.unwrap()]
                        })
                        .collect_vec();
                    graph.insert_patterns(
                        bundle
                    )
                } else {
                    let (pid, context_range, inner) = bundle.into_iter().next().unwrap();    
                    let context = graph.insert_range_in(
                        index.to_pattern_location(pid),
                        context_range,
                    ).unwrap();
                    if let Some(inner) = inner {
                        graph.insert_pattern(
                            [context, inner]
                        )
                    } else {
                        context
                    }
                }
            },
            Err(child) => child,
        }
    }
    pub fn last_child_partition(
        &mut self,
        cache: &SplitCache,
        prev_cache: &SplitPositionCache,
        index: &Child,
    ) -> Child {
        match self.last_partition(
            cache,
            prev_cache,
            index,
        ) {
            Ok(bundle) => {
                let mut graph = self.graph_mut();
                if bundle.len() > 1 {
                    let bundle = bundle.into_iter()
                        .map(|(pid, inner, context_range)| {
                            let context = graph.insert_range_in(
                                index.to_pattern_location(pid),
                                context_range,
                            ).unwrap();
                            vec![inner.unwrap(), context]
                        })
                        .collect_vec();
                    graph.insert_patterns(
                        bundle
                    )
                } else {
                    let (pid, inner, context_range) = bundle.into_iter().next().unwrap();    
                    let context = graph.insert_range_in(
                        index.to_pattern_location(pid),
                        context_range,
                    ).unwrap();
                    if let Some(inner) = inner {
                        graph.insert_pattern(
                            [inner, context]
                        )
                    } else {
                        context
                    }
                }
            },
            Err(child) => child,
        }
    }
    pub fn first_pattern_partition(
        &mut self,
        cache: &SplitCache,
        pos_cache: &SplitPositionCache,
        index: &Child,
    ) -> Pattern {
        match self.first_partition(
            cache,
            pos_cache,
            index,
        ) {
            Ok(bundle) => {
                let mut graph = self.graph_mut();
                if bundle.len() > 1 {
                    let bundle = bundle.into_iter()
                        .map(|(pid, context_range, inner)| {
                            let context = graph.insert_range_in(
                                index.to_pattern_location(pid),
                                context_range,
                            ).unwrap();
                            vec![context, inner.unwrap()]
                        })
                        .collect_vec();
                    vec![graph.insert_patterns(
                        bundle
                    )]
                } else {
                    let (pid, context_range, inner) = bundle.into_iter().next().unwrap();    
                    let context = graph.expect_child_pattern_range(
                        index.to_pattern_location(pid),
                        context_range,
                    );
                    if let Some(inner) = inner {
                        [context, &[inner]].concat()
                    } else {
                        context.to_vec()
                    }
                }
            },
            Err(child) => vec![child],
        }
    }
    pub fn last_pattern_partition(
        &mut self,
        cache: &SplitCache,
        prev_cache: &SplitPositionCache,
        index: &Child,
    ) -> Pattern {
        match self.last_partition(
            cache,
            prev_cache,
            index,
        ) {
            Ok(bundle) => {
                let mut graph = self.graph_mut();
                if bundle.len() > 1 {
                    let bundle = bundle.into_iter()
                        .map(|(pid, inner, context_range)| {
                            let context = graph.insert_range_in(
                                index.to_pattern_location(pid),
                                context_range,
                            ).unwrap();
                            vec![context, inner.unwrap()]
                        })
                        .collect_vec();
                    vec![graph.insert_patterns(
                        bundle
                    )]
                } else {
                    let (pid, inner, context_range) = bundle.into_iter().next().unwrap();    
                    let context = graph.expect_child_pattern_range(
                        index.to_pattern_location(pid),
                        context_range,
                    );
                    if let Some(inner) = inner {
                        [&[inner], context].concat()
                    } else {
                        context.to_vec()
                    }
                }
            },
            Err(child) => vec![child],
        }
    }
    pub fn partition_root(
        &mut self,
        cache: &mut SplitCache,
        index: &Child,
    ) -> RootPartitions {

        // 1. create partitions with all offsets
        //    creates smallest indices required by larger indices
        // 2. merge partitions in size ascending order into final splits
        //    make sure to include smaller new indices in larger ones

        let vert_cache = cache.entries.get(&index.index).unwrap();

        let num_offsets = vert_cache.positions.len();
        let mut offset_iter = vert_cache.positions.iter();

        let (parent_offset, pos_cache) = offset_iter.next().unwrap();
        match cache.root_mode {
            RootMode::Infix => {
                assert_eq!(num_offsets, 2);
                let prefix =
                    self.first_pattern_partition(
                        cache,
                        pos_cache,
                        index,
                    );
                let first_offset = pos_cache.find_clean_split();
                let (_prev_offset, prev_cache) = (parent_offset, pos_cache);
                let (parent_offset, pos_cache) = offset_iter.next().unwrap();
                let inner =
                    self.inner_partition(
                        cache,
                        prev_cache,
                        pos_cache,
                        index,
                    );

                let second_offset = pos_cache.find_clean_split();
                let (_prev_offset, prev_cache) = (parent_offset, pos_cache);
                let postfix =
                    self.last_pattern_partition(
                        cache,
                        prev_cache,
                        index,
                    );
                RootPartitions::Infix(
                    prefix,
                    first_offset,
                    inner,
                    second_offset,
                    postfix,
                )
            },
            RootMode::Prefix => {
                assert_eq!(num_offsets, 1);
                let inner =
                    self.first_child_partition(
                        cache,
                        pos_cache,
                        index,
                    );
                let offset = pos_cache.find_clean_split();
                let (_prev_offset, prev_cache) = (parent_offset, pos_cache);
                let postfix =
                    self.last_pattern_partition(
                        cache,
                        prev_cache,
                        index,
                    );
                RootPartitions::Prefix(
                    inner,
                    offset,
                    postfix,
                )
            },
            RootMode::Postfix => {
                assert_eq!(num_offsets, 1);
                let prefix =
                    self.first_pattern_partition(
                        cache,
                        pos_cache,
                        index,
                    );
                let offset = pos_cache.find_clean_split();
                let (_prev_offset, prev_cache) = (parent_offset, pos_cache);
                let inner =
                    self.last_child_partition(
                        cache,
                        prev_cache,
                        index,
                    );
                RootPartitions::Postfix(
                    prefix,
                    offset,
                    inner,
                )
            },
        }
    }
    pub fn partition_node(
        &mut self,
        cache: &mut SplitCache,
        index: &Child,
    ) -> Vec<Child> {

        // 1. create partitions with all offsets
        //    creates smallest indices required by larger indices
        // 2. merge partitions in size ascending order into final splits
        //    make sure to include smaller new indices in larger ones
        let graph = self.graph();
        let vert_cache = cache.entries.get(&index.index).unwrap();
        let patterns = graph.expect_child_patterns(index).clone();
        let offset_splits = vert_cache.positions.iter().map(|(off, cache)|
            (*off, cache.pattern_splits.clone())
        )
        .collect();
        drop(graph);
        self.index_offset_partitions(
            cache,
            *index,
            &patterns,
            &offset_splits,
        )
        .into_iter()
        .map(|part| part.index)
        .collect()

        //let mut offset_iter = vert_cache.positions.iter();
        //let (parent_offset, pos_cache) = offset_iter.next().unwrap();
        //let mut partitions = vec![
        //    self.first_child_partition(
        //        cache,
        //        pos_cache,
        //        index,
        //    )
        //];
        //let (mut prev_offset, mut prev_cache) = (parent_offset, pos_cache);
        //partitions.extend(
        //    offset_iter.map(|(parent_offset, pos_cache)| {
        //        let part = self.inner_partition(
        //            cache,
        //            prev_cache,
        //            pos_cache,
        //            index,
        //        );
        //        (prev_offset, prev_cache) = (parent_offset, pos_cache);
        //        part
        //    })
        //);
        //partitions.push(
        //    self.last_child_partition(
        //        cache,
        //        prev_cache,
        //        index,
        //    )
        //);
        //partitions
    }
    /// calls `index_range` for each offset partition
    fn index_offset_partitions(
        &mut self,
        cache: &SplitCache,
        index: Child,
        patterns: &HashMap<PatternId, Pattern>,
        offset_splits: &BTreeMap<NonZeroUsize, impl Borrow<PatternSubSplits>>,
    ) -> Vec<IndexedPartition> {
        assert!(offset_splits.len() > 1);
        let mut iter = offset_splits.iter()
            .map(|(&offset, splits)|
                OffsetSplitsRef {
                    offset,
                    splits: splits.borrow(),
                });
        let mut prev = iter.next().unwrap();
        iter.map(|offset| {
            let part = self.index_range(
                cache,
                index,
                patterns,
                (prev, offset)
            );
            prev = offset;
            part
        })
        .collect()
    }
    ///
    fn range_patterns<'a, O: AsOffsetSplits<'a>>(
        &mut self,
        cache: &SplitCache,
        index: Child,
        patterns: &HashMap<PatternId, Pattern>,
        range: (O, O),
    ) -> Result<(Vec<Pattern>, (Option<PatternId> ,Option<PatternId>)), IndexedPartition> {
        // collect infos about partition in each pattern
        match Self::range_partition_info(
            cache,
            patterns,
            range,
        ) {
            Ok(info) => {
                // index inner sub ranges
                let bundle = info.bundle.iter()
                    .map(|info| {
                        self.index_range_info(
                            cache,
                            index,
                            patterns,
                            info,
                        )
                    })
                    .collect_vec();
                    
                Ok((
                    bundle,
                    info.perfect,
                ))
            },
            Err(part) => Err(part),
        }
    }
    fn index_range<'a, O: AsOffsetSplits<'a>>(
        &mut self,
        cache: &SplitCache,
        index: Child,
        patterns: &HashMap<PatternId, Pattern>,
        range: (O, O),
    ) -> IndexedPartition {
        match self.range_patterns(
            cache,
            index,
            patterns,
            range,
        ) {
            Ok((bundle, perfect)) => {
                let mut graph = self.graph_mut();
                let index = graph.insert_patterns(
                    bundle
                );
                IndexedPartition {
                    index,
                    perfect,
                }
            }
            Err(part) => part,
        }
    }
    fn index_range_info(
        &mut self,
        cache: &SplitCache,
        index: Child,
        patterns: &HashMap<PatternId, Pattern>,
        info: &PatternPartitionInfo,
    ) -> Pattern {
        if let Some(context) = info.inner_range.as_ref().map(|range_info| {
            // index inner range
            let inner = self.index_range(
                cache,
                index,
                patterns,
                range_splits(patterns.iter(), range_info.offsets)
            );
            let mut graph = self.graph_mut();
            // replace range and with new index
            graph.insert_range_in(
                index.to_pattern_location(info.pattern_id),
                range_info.range.clone(),
            ).unwrap()
        }) {
            vec![info.left, context, info.right]
        } else {
            vec![info.left, info.right]
        }
    }
    fn range_partition_info<'a, O: AsOffsetSplits<'a>>(
        cache: &SplitCache,
        patterns: &HashMap<PatternId, Pattern>,
        range: (O, O),
    ) -> Result<PartitionBundle, IndexedPartition> {
        let (ls, rs) = (range.0.as_offset_splits(), range.1.as_offset_splits());
        let urange = (ls.offset.get(), rs.offset.get());
        let mut perfect = (None, None);
        ls.splits.iter().zip(rs.splits.iter()).map(|((&pid, prev_pos), (_, pos))| {
            // todo detect if prev offset is in same index (to use inner partition as result)
            let pattern = patterns.get(&pid).unwrap();

            let prev_child = pattern[prev_pos.sub_index];
            let prev_split = prev_pos.inner_offset.map(|inner_offset|
                cache.expect_final_split(&SplitKey::new(prev_child, inner_offset))
            );

            let child = pattern[pos.sub_index];
            let inner_split = pos.inner_offset.map(|inner_offset|
                cache.expect_final_split(&SplitKey::new(child, inner_offset))
            );

            // get split parts in this partition
            let context_range = prev_pos.sub_index+1..pos.sub_index;
            let context = pattern.get(prev_pos.sub_index+1..pos.sub_index).unwrap_or(&[]);
            let context_range = (!context.is_empty()).then(|| context_range);
            if prev_split.is_none() {
                perfect.0 = Some(pid);
            }
            if inner_split.is_none() {
                perfect.1 = Some(pid);
            }
            match (
                prev_pos.sub_index == pos.sub_index,
                prev_split,
                inner_split,
            ) {
                (true, Some(prev_split), Some(inner_split)) => {
                    // todo find inner partition
                    unimplemented!();
                    //self.get_partition(
                    //    merges,
                    //    offsets,
                    //    range,
                    //)
                    Err(IndexedPartition {
                        index: inner_split.left,
                        perfect,
                    })
                },
                (true, None, Some(inner_split)) => {
                    // todo find inner partition
                    unimplemented!();
                    Err(IndexedPartition {
                        index: inner_split.left,
                        perfect,
                    })
                },
                (true, Some(prev_split), None) => {
                    unreachable!("Invalid split position or invalid offset order");
                },
                (true, None, None) =>
                    Err(IndexedPartition {
                        index: child,
                        perfect,
                    }),
                (false, Some(prev_split), Some(inner_split)) =>
                    Ok(PatternPartitionInfo {
                        pattern_id: pid,
                        left: prev_split.right,
                        right: inner_split.left,
                        inner_range: context_range.map(|range|
                            InnerRangeInfo {
                                range,
                                offsets: to_non_zero_range(
                                    urange.0 + prev_split.right.width(),
                                    urange.1 - inner_split.left.width()
                                ),
                            }
                        ),
                    }),
                (false, None, Some(inner_split)) =>
                    Ok(PatternPartitionInfo {
                        pattern_id: pid,
                        left: prev_child,
                        right: inner_split.left,
                        inner_range: context_range.map(|range|
                            InnerRangeInfo {
                                range,
                                offsets: to_non_zero_range(
                                    urange.0 + prev_child.width(),
                                    urange.1 - inner_split.left.width()
                                ),
                            }
                        ),
                    }),
                (false, Some(prev_split), None) =>
                    Ok(PatternPartitionInfo {
                        pattern_id: pid,
                        left: prev_split.right,
                        right: child,
                        inner_range: context_range.map(|range|
                            InnerRangeInfo {
                                range,
                                offsets: to_non_zero_range(
                                    urange.0 + prev_split.right.width(),
                                    urange.1 - child.width()
                                ),
                            }
                        ),
                    }),
                (false, None, None) =>
                    Ok(PatternPartitionInfo {
                        pattern_id: pid,
                        left: prev_child,
                        right: child,
                        inner_range: context_range.map(|range|
                            InnerRangeInfo {
                                range,
                                offsets: to_non_zero_range(
                                    urange.0 + prev_child.width(),
                                    urange.1 - child.width()
                                ),
                            }
                        ),
                    }),

            }
        })
        .collect::<Result<Vec<_>, _>>()
        .map(|bundle|
            PartitionBundle {
                bundle,
                perfect,
            }
        )
    }
    //fn index_ranges(
    //    &mut self,
    //    cache: &SplitCache,
    //    index: Child,
    //    patterns: &HashMap<PatternId, Pattern>,
    //    ranges: &Vec<(OffsetSplits, OffsetSplits)>,
    //) -> Vec<Child> {
    //    let mut inner_offsets = BTreeMap::new();
    //    let infos = ranges.iter()
    //        .flat_map(|(l, r)|
    //            self.range_partition_info(
    //                cache,
    //                index,
    //                patterns,
    //                (l, r),
    //            )
    //        )
    //        .map(|(l, r)| {
    //            inner_offsets.insert(l.offset, l.splits);
    //            inner_offsets.insert(r.offset, r.splits);
    //            (l.offset, r.offset)
    //        })
    //        .collect();
    //    let partitions = self.index_offset_partitions(
    //        cache,
    //        index,
    //        patterns,
    //        &inner_offsets,
    //    );
    //}
    fn get_partition(
        &mut self,
        merges: &HashMap<Range<usize>, Child>,
        offsets: &Vec<Offset>,
        range: &Range<usize>,
    ) -> Option<Child> {
        let graph = self.graph();
        //let split_map: BTreeMap<_, Split<Option<Child>>> = Default::default();

        let wrapper = merges.get(range)?;
        Some(if range.start == range.end {
            *wrapper
        } else {
            let pre_width = range.start.checked_sub(1)
                .map(|prev| NonZeroUsize::new(
                    offsets[range.start].get() - offsets[prev].get()
                ).unwrap())
                .unwrap_or(offsets[range.start]);

            let wrapper = merges.get(range)?;
            let node = graph.expect_vertex_data(wrapper);

            let (_, pat) = node.get_child_pattern_with_prefix_width(pre_width).unwrap();

            let wrapper2 = pat[1];
            let node2 = graph.expect_vertex_data(wrapper2);


            let inner_width = NonZeroUsize::new(offsets[range.end].get() - offsets[range.start].get()).unwrap();
            let (_, pat2) = node2.get_child_pattern_with_prefix_width(inner_width).unwrap();
            pat2[0]
        })
    }
    //fn inner_ranges_offset_splits(
    //    patterns: &HashMap<PatternId, Pattern>,
    //    range: &(OffsetSplits, OffsetSplits),
    //) -> Vec<(OffsetSplits, OffsetSplits)> {
    //    // find offsets for inner ranges
    //    patterns.iter()
    //        .filter_map(|(pid, p)| {
    //            Self::inner_range_offset_splits(
    //                patterns,
    //                pid,
    //                p,
    //                range,
    //            )
    //        })
    //        .collect()
    //}
    // find split locations for each inner range of each pattern, if any
    //fn inner_range_offset_splits(
    //    patterns: &HashMap<PatternId, Pattern>,
    //    &pid: &PatternId,
    //    pattern: &Pattern,
    //    range: &(OffsetSplits, OffsetSplits),
    //) -> Option<(OffsetSplits, OffsetSplits)> {
    //    let l = range.0.splits.get(&pid).unwrap();
    //    let r = range.1.splits.get(&pid).unwrap();
    //    let u_range = (range.0.offset.get(), range.1.offset.get());
    //    match (&l.inner_offset, &r.inner_offset) {
    //        (Some(lo), Some(ro)) =>
    //            (r.sub_index - l.sub_index > 2).then_some(
    //                (
    //                    l.sub_index + 1,
    //                    u_range.0 + lo.get(),
    //                    r.sub_index,
    //                    u_range.1 - ro.get(),
    //                )
    //            ),
    //        (None, None) =>
    //            (r.sub_index - l.sub_index > 4).then_some(
    //                (
    //                    l.sub_index + 1,
    //                    u_range.0 + pattern[l.sub_index].width(),
    //                    r.sub_index - 1,
    //                    u_range.1 - pattern[r.sub_index-1].width(),
    //                )
    //            ),
    //        (None, Some(ro)) =>
    //            (r.sub_index - l.sub_index > 2).then_some(
    //                (
    //                    l.sub_index + 1,
    //                    u_range.0 + pattern[l.sub_index].width(),
    //                    r.sub_index,
    //                    u_range.1 + ro.get(),
    //                )
    //            ),
    //        (Some(lo), None) =>
    //            (r.sub_index - l.sub_index > 3).then_some(
    //                (
    //                    l.sub_index + 1,
    //                    u_range.0 + lo.get(),
    //                    r.sub_index - 1,
    //                    u_range.1 - pattern[r.sub_index-1].width(),
    //                )
    //            ),
    //    }.map(|(li, lo, ri, ro)| {
    //        // find splits for other patterns
    //        let (lo, ro) = (
    //            NonZeroUsize::new(lo).unwrap(),
    //            NonZeroUsize::new(ro).unwrap(),
    //        );
    //        let mut ls =
    //            position_splits(
    //                patterns.iter().filter(|(id, _)| **id != pid),
    //                lo,
    //            );
    //        ls.insert(
    //            pid,
    //            PatternSplitPos {
    //                sub_index: li,
    //                inner_offset: None,
    //            },
    //        );
    //        let mut rs =
    //            position_splits(
    //                patterns.iter().filter(|(id, _)| **id != pid),
    //                ro,
    //            );
    //        rs.insert(
    //            pid,
    //            PatternSplitPos {
    //                sub_index: ri,
    //                inner_offset: None,
    //            },
    //        );
    //        (
    //            OffsetSplits {
    //                offset: lo,
    //                splits: ls,
    //            },
    //            OffsetSplits {
    //                offset: ro,
    //                splits: rs,
    //            },
    //        )
    //    })
    //}
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
#[derive(Debug)]
pub struct PatternPartitionInfo {
    pub pattern_id: PatternId,
    pub inner_range: Option<InnerRangeInfo>,
    pub left: Child,
    pub right: Child,
}
#[derive(Debug)]
pub struct PartitionBundle {
    pub bundle: Vec<PatternPartitionInfo>,
    pub perfect: (Option<PatternId>, Option<PatternId>),
}
#[derive(Debug)]
pub struct IndexedPartition {
    pub index: Child,
    pub perfect: (Option<PatternId>, Option<PatternId>),
}
#[derive(Debug)]
pub struct IndexedPattern {
    pub pattern: Pattern,
    pub perfect: (Option<PatternId>, Option<PatternId>),
}
#[derive(Debug)]
pub struct InnerRangeInfo {
    pub range: Range<usize>,
    pub offsets: (NonZeroUsize, NonZeroUsize),
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
trait AsOffsetSplits<'a>: 'a {
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
impl<'a> AsOffsetSplits<'a> for OffsetSplitsRef<'a> {
    fn as_offset_splits<'t>(&'t self) -> OffsetSplitsRef<'t> where 'a: 't {
        *self
    }
}