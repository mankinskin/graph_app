use crate::*;

#[derive(Debug, Clone)]
pub struct Pather<Side: IndexSide<<BaseGraphKind as GraphKind>::Direction>> {
    pub(crate) indexer: Indexer,
    _ty: std::marker::PhantomData<Side>,
}
impl<Side: IndexSide<<BaseGraphKind as GraphKind>::Direction>> Pather<Side> {
    pub fn new(indexer: Indexer) -> Self {
        Self {
            indexer,
            _ty: Default::default()
        }
    }
}
#[derive(Debug, Clone)]
pub struct IndexPathComponents {
    prev_primary: Child,
    primary_exclusive: Option<Child>,
    prev_secondary: Child,
    location: ChildLocation,
    prev_location: ChildLocation,
}
//#[derive(Debug, Clone)]
//struct PathFeatures {
//    len: usize,
//    has_primary_exclusive: bool,
//}
//#[derive(Debug, Clone)]
//enum IndexingMode {
//    /// create new index wrapping primary part and residual secondary parts
//    Wrapping,
//    /// postpone indexing until it is needed and more information is available
//    Postpone,
//    /// insert local pattern with indexed primary and secondary halves
//    Local,
//}
//impl PathFeatures {
//    fn is_full_primary(&self) -> bool {
//        !self.has_primary_exclusive && self.len == 1
//    }
//    pub fn new<
//        D: IndexDirection,
//        Side: IndexSide<D>,
//        S: RelativeSide<D, Side>,
//    >(
//        path: &Vec<ChildLocation>,
//    ) -> Self {
//        assert!(!path.is_empty());
//        let len = path.len();
//        let path = Side::bottom_up_path_iter(path.iter());
//        let last = path.last().unwrap();
//        PathFeatures {
//            has_primary_exclusive: <S as RelativeSide<D, Side>>::exclusive_primary_location(last).is_some(),
//            len,
//        }
//    }
//}

impl<Side: IndexSide<<BaseGraphKind as GraphKind>::Direction>> Pather<Side> {
    #[instrument(skip(self, entry), ret)]
    pub fn index_primary_entry<
        S: RelativeSide<Side>,
        L: Borrow<ChildLocation> + Debug,
    >(
        &mut self,
        entry: L,
    ) -> Option<(Pattern, IndexSplitResult)> {
        let pattern = self.graph().expect_pattern_at(entry.borrow()).clone();
        self.pattern_perfect_split::<S, _, _>(
            pattern,
            entry,
        )
    }
    //pub fn index_primary_path_bundle<
    //    S: RelativeSide<D, Side>,
    //>(
    //    &mut self,
    //    bundle: Vec<Vec<ChildLocation>>,
    //) -> Option<IndexSplitResult> {
    //    let features = bundle.iter()
    //        .map(|path| PathFeatures::new::<_, _, S>(path))
    //        .collect_vec();
    //    let zip = bundle.iter().zip(features.iter());
    //    if let Some(path) = zip.clone().find_map(|(p, f)| f.is_full_primary().then(|| p)) {
    //        // some path to full primary
    //        self.index_primary_path(path)
    //        // might still need to use other paths for indexing prev_secondary in consequtive indexing
    //    } else {
    //        if let Some(path) = zip.find_map(|(p, f)| (!f.has_primary_exclusive).then(|| p)) {
    //            // primary given by prev_primary
    //            Some(IndexSplitResult {
    //                inner: prev_primary,
    //                location: location,
    //                path: vec![prev_location],
    //            })
    //        } else {
    //            // none full primary, need to create new index
    //            let components = bundle.into_iter()
    //                .map(|path| self.path_components(path).unwrap())
    //                .collect_vec();
    //            let primary = self.graph_mut().insert_patterns(
    //                components.iter().map(|components| {
    //                    let (back, front) = S::outer_inner_order(components.primary_exclusive.unwrap(), components.prev_primary);
    //                    [back, front]
    //                })
    //            );
    //            let secondary = self.graph_mut().insert_patterns(
    //                components.iter().map(|components| {
    //                    let (back, front) = S::outer_inner_order(components.primary_exclusive.unwrap(), components.prev_primary);
    //                    [back, front]
    //                })
    //            );
    //            None
    //        }
    //    }
    //}

    pub fn path_components<
        S: RelativeSide<Side>,
        P: ContextPath
    >(
        &mut self,
        path: P,
    ) -> Option<IndexPathComponents> {
        let mut iter = Side::bottom_up_path_iter(path);
        let entry = iter.next()?;
        let (_, mut prev) = self.index_primary_entry::<S, _>(entry)?;
        let last = if !iter.is_empty() {
            while iter.len() > 1 {
                let location = iter.next().unwrap();
                prev = self.path_segment::<S>(prev, location);
            }
            iter.next().unwrap()
        } else {
            entry
        };
        Some(self.path_entry_components::<S>(
            prev,
            last,
        ))
    }
    pub fn path_entry_components<
        S: RelativeSide<Side>,
    >(
        &mut self,
        prev: IndexSplitResult,
        location: ChildLocation
    ) -> IndexPathComponents {
        let IndexSplitResult {
            inner: prev_primary,
            location: prev_location,
            path: mut prev_path
        } = prev;
        // todo: make sure to append loc on correct side
        prev_path.push(prev_location.clone());
        let IndexSplitResult {
            inner: prev_secondary,
            ..
        } = self.index_secondary_path::<S, _>(prev_path).unwrap();

        // todo:
        // - return whether the exclusive part was more than one index
        // - use missing exclusive part for later logic
        // - if primary exclusive part is missing, use prev_primary for a
        IndexPathComponents {
            prev_primary,
            primary_exclusive:
                self.index_primary_exclusive::<S, _>(location)
                    .map(|(_, primary_exclusive)| primary_exclusive.inner),
            prev_secondary,
            location,
            prev_location,
        }
    }
    pub fn index_components<
        S: RelativeSide<Side>,
    >(
        &mut self,
        components: IndexPathComponents,
    ) -> IndexSplitResult {
        let IndexPathComponents {
            prev_primary,
            primary_exclusive,
            prev_secondary,
            location,
            prev_location,
        } = components;

        let has_primary_exclusive = primary_exclusive.is_some();

        let primary = primary_exclusive.map(|primary_exclusive|
            S::index_inner_and_context(&mut self.indexer, prev_primary, primary_exclusive)
        ).unwrap_or(prev_primary);

        let (back, front) = S::outer_inner_order(primary, prev_secondary);
        // simply wrap and replace old range with new primary split
        let pattern = self.graph().expect_pattern_at(location.borrow()).clone();
        match (S::has_secondary_exclusive(&pattern, location.sub_index), has_primary_exclusive) {
            (true, true) => {
                // with secondary exclusive
                // with primary exclusive
                let range = S::primary_range(location.sub_index);
                let (wrapper, pids) = self.graph_mut().insert_patterns_with_ids([
                    &pattern[range.clone()],
                    &[back, front][..],
                ]);
                self.graph_mut().replace_in_pattern(location, range, [wrapper]);
                IndexSplitResult {
                    inner: primary,
                    location: location.to_child_location(S::primary_indexed_pos(location.sub_index)),
                    path: vec![
                        ChildLocation {
                            parent: wrapper,
                            pattern_id: pids[1],
                            sub_index: 1,
                        }
                    ],
                }
            },
            (false, true) => {
                // no secondary exclusive
                let pid = self.graph_mut().add_pattern_with_update(
                    location,
                    [back, front],
                );
                IndexSplitResult {
                    inner: primary,
                    location: location
                        .to_pattern_location(pid)
                        .to_child_location(1),
                    path: vec![],
                }
            },
            (_, false) => {
                // with secondary exclusive
                // no primary exclusive
                // point to previous primary
                IndexSplitResult {
                    inner: prev_primary,
                    location: location,
                    path: vec![prev_location],
                }
            }
        }
    }
    pub fn path_segment<
        S: RelativeSide<Side>,
    >(
        &mut self,
        prev: IndexSplitResult,
        location: ChildLocation
    ) -> IndexSplitResult {
        let components = self.path_entry_components::<S>(prev, location);
        self.index_components::<S>(
            components,
        )
    }
    #[instrument(skip(self, path))]
    //#[async_recursion]
    pub fn index_primary_path<
        S: RelativeSide<Side>,
        P: ContextPath,
    >(
        &mut self,
        path: P,
    ) -> Option<IndexSplitResult> {
        let mut iter = Side::bottom_up_path_iter(path);
        let entry  = iter.next()?;
        let (_, mut prev) = self.index_primary_entry::<S, _>(entry)?;
        while let Some(location) = iter.next() {
            prev = self.path_segment::<S>(prev, location);
        }
        Some(prev)
    }
    // primary range, depending on S
    fn index_primary_exclusive<
        S: RelativeSide<Side>,
        L: Borrow<ChildLocation> + Debug,
    >(
        &mut self,
        location: L,
    ) -> Option<(Pattern, IndexSplitResult)> {
        self.index_primary_entry::<S, _>(
            <S as RelativeSide<Side>>::exclusive_primary_location(*location.borrow())?
        )
    }
    pub fn index_secondary_path<
        S: RelativeSide<Side>,
        P: ContextPath
    >(
        &mut self,
        path: P,
    ) -> Option<IndexSplitResult> {
        self.index_primary_path::<<S as RelativeSide<Side>>::Opposite, _>(
            path,
        )
    }
    /// index inner half of pattern
    /// 
    /// Creates an index for the inner half of a pattern if needed and returns the new index
    /// along with its location
    #[instrument(skip(self))]
    fn pattern_perfect_split<
        S: RelativeSide<Side>,
        P: IntoPattern,
        L: Borrow<ChildLocation> + Debug
    >(
        &mut self,
        pattern: P,
        location: L,
    ) -> Option<(Pattern, IndexSplitResult)> {
        let location = location.borrow();
        info!("first split");
        let secondary = S::split_secondary(&pattern, location.sub_index);
        if secondary.is_empty() {
            None
        } else {
            let range = S::primary_range(location.sub_index);
            let primary = &pattern.borrow()[range.clone()];
            let primary = if primary.len() < 2 {
                *primary.iter().next()?
            } else {
                let mut graph = self.graph_mut();
                let primary = graph.insert_pattern(primary);
                graph.replace_in_pattern(location, range.clone(), [primary]);
                primary
            };
            Some((
                secondary.to_vec(),
                IndexSplitResult {
                    location: location.to_child_location(S::primary_indexed_pos(range.start())),
                    path: vec![],
                    inner: primary,
                }
            ))
        }
    }
    #[instrument(skip(self, parent, child_patterns, offset))]
    //#[async_recursion]
    fn child_pattern_offset_splits<
        S: RelativeSide<Side>,
    >(
        &mut self,
        parent: Child,
        child_patterns: ChildPatterns,
        offset: NonZeroUsize,
    ) -> Result<IndexSplitResult, Vec<(ChildLocation, Pattern, IndexSplitResult, Child)>> {
        let mut child_patterns = child_patterns.into_iter();
        let len = child_patterns.len();
        match child_patterns
            .try_fold(Vec::with_capacity(len), |mut acc, (pid, pattern)| {
                let (index, inner_offset) = Side::token_offset_split(
                    pattern.borrow() as &[Child],
                    offset,
                ).unwrap();
                if let Some(inner_offset) = inner_offset {
                    acc.push((pid, pattern, index, inner_offset));
                    ControlFlow::Continue(acc)
                } else {
                    ControlFlow::Break((pattern.into_pattern(), pid, index))
                }
            })
        {
            ControlFlow::Break((pattern, pid, pos)) =>
                Ok(
                    self.pattern_perfect_split::<S, _, _>(
                        pattern,
                        ChildLocation::new(parent, pid, pos),
                    ).expect("Offset non-zero!").1,
                ),
            ControlFlow::Continue(c) => {
                let pather = &self.clone();
                Err(
                    c.into_iter()
                        .map(|(pid, pattern, pos, offset)| {
                            let mut pather = pather.clone();
                            let sub = *pattern.get(pos).unwrap();
                            // split index at pos with offset
                            let split = pather.single_offset_split::<S>(
                                sub,
                                offset,
                            );

                            // index inner context
                            let IndexSplitResult {
                                inner: context,
                                ..
                            } = pather.index_secondary_path::<S, _>(
                                split.path.clone().into_iter().chain(
                                    std::iter::once(split.location)
                                ).collect_vec(),
                            ).unwrap();
                            (parent.to_child_location(SubLocation::new(pid, pos)), pattern, split, context)
                        })
                    .collect()
                        
                )
            },
        }
    }
    /// split parent at token offset from direction start
    #[instrument(skip(self, parent, offset))]
    //#[async_recursion]
    pub fn single_offset_split<
        S: RelativeSide<Side>,
    >(
        &mut self,
        parent: Child,
        offset: NonZeroUsize,
    ) -> IndexSplitResult {
        if offset.get() >= parent.width() {
            assert!(offset.get() < parent.width());
        }
        let child_patterns = self.graph().expect_child_patterns(&parent).clone();
        // find perfect split in parent
        match self.child_pattern_offset_splits::<S>(
            parent,
            child_patterns,
            offset
        ) {
            Ok(split) => split,
            Err(splits) =>
                self.unperfect_splits(
                    parent,
                    splits,
                ),
        }
    }
    #[instrument(skip(self, location, split, split_ctx))]
    fn entry_unperfect_split(
        &mut self,
        location: ChildLocation,
        split: IndexSplitResult,
        split_ctx: Child,
    ) -> IndexSplitResult {
        let mut graph = self.graph_mut();
        // split index at pos with offset
        let IndexSplitResult {
            inner,
            path: _,
            location: split_location,
        } = split;
        let pos = location.sub_index;
        // inner part of child pattern (context of split index)
        if let Some(parent_inner) = graph.insert_range_in(
                location,
                Side::inner_context_range(pos)
            ).ok()
        {
            // split_inner + split inner context
            let full_inner = graph.insert_pattern(
                // context on opposite side than usual (inner side)
                <Side as IndexSide<_>>::Opposite::concat_inner_and_context(inner, parent_inner),
            );
            // ||    |     ||      |
            //       ^^^^^^^^^^^^^^
            // index for inner half including split
            if let Ok(wrapper) = graph.insert_range_in(
                location,
                Side::inner_range(pos),
            ) {
                // more context before split, need wrapper
                let wrapper_pid = graph.add_pattern_with_update(
                    wrapper,
                    Side::concat_inner_and_context(full_inner, split_ctx),
                );
                graph.replace_in_pattern(
                    split_location,
                    Side::inner_range(pos),
                    wrapper,
                );
                IndexSplitResult {
                    location,
                    path: vec![
                        ChildLocation::new(inner, wrapper_pid, 1),
                    ],
                    inner: full_inner,
                }
            } else {
                // no context before split
                let pid = graph.add_pattern_with_update(
                    location.parent,
                    Side::concat_inner_and_context(full_inner, split_ctx),
                );
                let (pos, _) = Side::back_front_order(0, 1);
                IndexSplitResult {
                    location: ChildLocation::new(location.parent, pid, pos),
                    path: vec![],
                    inner: full_inner,
                }
            }
        } else {
            // no inner context
            IndexSplitResult {
                location,
                path: vec![
                    split_location
                ],
                inner,
            }
        }
    }
    #[instrument(skip(self, parent, splits))]
    fn unperfect_splits(
        &mut self,
        parent: Child,
        splits: Vec<(ChildLocation, Pattern, IndexSplitResult, Child)>,
    ) -> IndexSplitResult {
        if splits.len() == 1 {
            let (location, _, split, context) = splits.into_iter().next().unwrap();
            self.entry_unperfect_split(
                location,
                split,
                context,
            )
        } else {
            // add contexts
            let mut backs = HashSet::default();
            let mut fronts: HashSet<Pattern> = HashSet::default();
            for (location, pattern, split, context) in splits {
                let pos = location.sub_index;
                let IndexSplitResult {
                    inner,
                    path: _,
                    location: _,
                } = split;
                let (back, front) = Side::context_inner_order(&context, &inner);
                // todo: order depends on D
                backs.insert([&Start::back_context(pattern.borrow(), pos)[..], back].concat());
                //fronts.insert([front, &End::front_context(pattern.borrow(), pos)[..]].concat());
            }
            
            // index half patterns
            let mut graph = self.graph_mut();
            let (back, front) = (
                graph.insert_patterns(backs),
                graph.insert_patterns(fronts),
            );
            let pid = graph.add_pattern_with_update(parent, [back, front]);
            // todo: order depends on D
            let (inner, _) = Side::back_front_order(back, front);
            let (pos, _) = Side::back_front_order(0, 1);
            let location = ChildLocation::new(parent, pid, pos);
            IndexSplitResult {
                location,
                path: vec![],
                inner,
            }
        }
    }
}