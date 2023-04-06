use crate::*;
pub enum RootPartitions {
    Prefix(Child, Pattern),
    Postfix(Pattern, Child),
    Inner(Pattern, Child, Pattern),
}
impl RootPartitions {
    pub fn inner(&self) -> &Child {
        match self {
            Self::Inner(_, inner, _) => inner,
            Self::Prefix(inner, _) => inner,
            Self::Postfix(_, inner) => inner,
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
                (None, None) => unreachable!(), //Err(child),
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
                let (_prev_offset, prev_cache) = (parent_offset, pos_cache);
        
                let (parent_offset, pos_cache) = offset_iter.next().unwrap();
                let inner =
                    self.inner_partition(
                        cache,
                        prev_cache,
                        pos_cache,
                        index,
                    );

                let (_prev_offset, prev_cache) = (parent_offset, pos_cache);
                let postfix =
                    self.last_pattern_partition(
                        cache,
                        prev_cache,
                        index,
                    );
                RootPartitions::Inner(
                    prefix,
                    inner,
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
                let (_prev_offset, prev_cache) = (parent_offset, pos_cache);
                let postfix =
                    self.last_pattern_partition(
                        cache,
                        prev_cache,
                        index,
                    );
                RootPartitions::Prefix(
                    inner,
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
                let (_prev_offset, prev_cache) = (parent_offset, pos_cache);

                let inner =
                    self.last_child_partition(
                        cache,
                        prev_cache,
                        index,
                    );
                RootPartitions::Postfix(
                    prefix,
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

        let vert_cache = cache.entries.get(&index.index).unwrap();

        let mut offset_iter = vert_cache.positions.iter();
        let (parent_offset, pos_cache) = offset_iter.next().unwrap();
        let mut partitions = vec![
            self.first_child_partition(
                cache,
                pos_cache,
                index,
            )
        ];
        let (mut prev_offset, mut prev_cache) = (parent_offset, pos_cache);
        partitions.extend(
            offset_iter.map(|(parent_offset, pos_cache)| {
                let part = self.inner_partition(
                    cache,
                    prev_cache,
                    pos_cache,
                    index,
                );
                (prev_offset, prev_cache) = (parent_offset, pos_cache);
                part
            })
        );
        partitions.push(
            self.last_child_partition(
                cache,
                prev_cache,
                index,
            )
        );
        partitions
    }
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
    pub fn merge_partitions(
        &mut self,
        partitions: &Vec<Child>,
    ) -> HashMap<Range<usize>, Child> {
        let mut graph = self.graph_mut();
        //let split_map: BTreeMap<_, Split<Option<Child>>> = Default::default();

        // this will contain all future indices
        let mut range_map = HashMap::default();

        let num_offsets = partitions.len() - 1;
        for (i, part) in partitions.iter().enumerate() {
            range_map.insert(
                i..i, 
                *part,
            );
        }
        for len in 1..num_offsets {
            for start in 0..num_offsets-len+1 {
                let range = start..start + len;
                range_map.insert(
                    range,
                    graph.insert_patterns(
                        (start..start + len).into_iter()
                            .map(|ri| {
                                let &left = range_map.get(&(start..ri))
                                    .unwrap();
                                    //.unwrap_or(&partitions[start]);
                                let &right = range_map.get(&(ri..start + len))
                                    .unwrap();
                                    //.unwrap_or(&partitions[start + len]);
                                vec![left, right]
                            })
                    ),
                );
            } 
        } 
        range_map
    }
    //pub fn find_ranges<
    //    'a,
    //>(
    //    &mut self,
    //    offset_caches: &mut HashMap<NonZeroUsize, SplitPositionCache>,
    //    root: VertexIndex,
    //) -> RangeMap {
    //    // maybe only for inner node

    //    // range -> child partitions
    //    // range -> offsets
    //    // offset -> parent ranges

    //    // 1. iterate ranges
    //    //      1.2. fill top edges
    //    //      1.3. add child partitions
    //    // 2. 

    //    let mut graph = self.graph_mut();
    //    let node = graph.expect_vertex_data(root);

    //    let offsets: BTreeSet<NonZeroUsize> = offset_caches.keys().cloned().collect();
    //    let num_offsets = offsets.len();
    //    let mut queue = VecDeque::new();

    //    for len in 1..num_offsets {
    //        for start in 0..num_offsets-len+1 {
    //            queue.push_back(
    //                offsets.iter().nth(start).unwrap().get()
    //                ..
    //                offsets.iter().nth(start + len).unwrap().get()
    //            );
    //        }
    //    }

    //    let mut ranges = HashMap::default();
    //    for range in queue {
    //        queue.extend(
    //            self.range_children(
    //                &offsets,
    //                &node.children,
    //                &mut ranges,
    //                range,
    //            )
    //        );
    //    }
    //    ranges
    //}
    //pub fn range_children<
    //    'a,
    //>(
    //    &mut self,
    //    offsets: &BTreeSet<NonZeroUsize>,
    //    patterns: &ChildPatterns,
    //    ranges: &mut HashMap<Range<usize>, PartitionCache>,
    //    range: Range<usize>,
    //) {
    //    // find inner ranges in original patterns
    //    let inner_ranges = Self::inner_ranges(
    //        patterns.iter(),
    //        range,
    //    );

    //    // for new ranges:
    //    // find overlaps with other ranges
    //}
    //pub fn inner_ranges<'a>(
    //    patterns: impl Iterator<Item=(&'a PatternId, &'a Pattern)>,
    //    range: Range<usize>,
    //) -> Vec<Range<usize>> {
    //    patterns
    //        .filter_map(|(pid, pat)| { 
    //            let (li, left_offset) = <IndexBack as IndexSide<Right>>::token_offset_split(
    //                pat.borrow(),
    //                range.start,
    //            ).unwrap();
    //            let (ri, right_offset) = <IndexBack as IndexSide<Right>>::token_offset_split(
    //                pat.borrow(),
    //                range.end,
    //            ).unwrap();
    //            (ri - li > 3).then(||
    //                range.start + pat[li].width() - left_offset.map(NonZeroUsize::get).unwrap_or_default()
    //                ..
    //                range.end - right_offset.map(NonZeroUsize::get).unwrap_or_default()
    //            )
    //        })
    //        .collect()
    //}
}
pub type RangeMap = HashMap<Range<usize>, PartitionCache>;

#[derive(Debug)]
pub enum PartitionCache {
    SubSplits(PatternSubSplits),
}