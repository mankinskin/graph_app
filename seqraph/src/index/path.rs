use crate::*;

type HashSet<T> = DeterministicHashSet<T>;
type HashMap<K, V> = DeterministicHashMap<K, V>;

#[derive(Debug, Clone)]
pub struct Pather<T: Tokenize, D: IndexDirection, Side: IndexSide<D>> {
    indexer: Indexer<T, D>,
    _ty: std::marker::PhantomData<(D, Side)>,
}
impl<'a: 'g, 'g, T: Tokenize + 'a, D: IndexDirection + 'a, Side: IndexSide<D>> Pather<T, D, Side> {
    pub fn new(indexer: Indexer<T, D>) -> Self {
        Self {
            indexer,
            _ty: Default::default()
        }
    }
}
impl<'a: 'g, 'g, T: Tokenize + 'a, D: IndexDirection + 'a, Side: IndexSide<D> + 'a> Traversable<'a, 'g, T> for Pather<T, D, Side> {
    type Guard = RwLockReadGuard<'g, Hypergraph<T>>;
    fn graph(&'g self) -> Self::Guard {
        self.indexer.graph()
    }
}
impl<'a: 'g, 'g, T: Tokenize + 'a, D: IndexDirection + 'a, Side: IndexSide<D> + 'a> TraversableMut<'a, 'g, T> for Pather<T, D, Side> {
    type GuardMut = RwLockWriteGuard<'g, Hypergraph<T>>;
    fn graph_mut(&'g mut self) -> Self::GuardMut {
        self.indexer.graph_mut()
    }
}
impl<'a: 'g, 'g, T: Tokenize + 'a, D: IndexDirection + 'a, Side: IndexSide<D>> Pather<T, D, Side> {
    #[instrument(skip(self, entry), ret)]
    pub fn index_primary_entry<
        S: RelativeSide<D, Side>,
        L: Borrow<ChildLocation> + Debug,
    >(
        &'a mut self,
        entry: L,
    ) -> Option<(Pattern, IndexSplitResult)> {
        let pattern = self.graph().expect_pattern_at(entry.borrow());       
        self.pattern_perfect_split::<S, _, _>(
            pattern,
            entry,
        )
    }
    // primary range, depending on S
    fn index_primary_exclusive<
        S: RelativeSide<D, Side>,
        L: Borrow<ChildLocation> + Debug,
    >(
        &'a mut self,
        location: L,
    ) -> Option<(Pattern, IndexSplitResult)> {
        self.index_primary_entry::<S, _>(
            <S as RelativeSide<D, Side>>::exclusive_primary_location(*location.borrow())?
        )
    }
    pub fn index_secondary_path<
        S: RelativeSide<D, Side>,
        P: ContextPath
    >(
        &'a mut self,
        path: P,
    ) -> Option<IndexSplitResult> {
        self.index_primary_path::<<S as RelativeSide<D, Side>>::Opposite, _>(
            path,
        )
    }
    #[instrument(skip(self, path))]
    pub fn index_primary_path<
        S: RelativeSide<D, Side>,
        P: ContextPath
    >(
        &'a mut self,
        path: P,
    ) -> Option<IndexSplitResult> {
        let mut iter = Side::bottom_up_path_iter(path);
        let entry  = iter.next()?;
        let (_, prev) = self.index_primary_entry::<S, _>(entry)?;
        iter.try_fold(prev, |
            IndexSplitResult {
                inner: prev_primary,
                location: prev_loc,
                path: prev_path
            },
            location
        | {
            let location = location.borrow();

            let IndexSplitResult {
                inner: prev_secondary,
                ..    
            }= self.index_secondary_path::<S, _>(
                prev_path.into_iter().chain(std::iter::once(prev_loc))
            ).unwrap();

            let primary = self.index_primary_exclusive::<S, _>(location)
                    .map(|(_, r)| 
                        S::index_inner_and_context(&mut self.indexer, prev_primary, r.inner)
                    )
                    .unwrap_or(prev_primary);
            let offset = Side::inner_width_to_offset(
                &location.parent,
                primary.width(),
            ).unwrap();
            let (back, front) = S::outer_inner_order(primary, prev_secondary);
            let child_patterns = self.graph().expect_child_patterns(location.parent).clone();
            assert!(!child_patterns.is_empty());
            Some(if child_patterns.len() == 1 {
                // simply wrap and replace old range with new primary split
                let pattern = self.graph().expect_pattern_at(location.borrow());       
                if !S::in_context() && S::split_secondary(&pattern, location.sub_index).is_empty()
                    || S::in_context() && S::split_primary(&pattern, location.sub_index).len() == 1
                {
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
                } else if S::split_primary(&pattern, location.sub_index).len() > 1 {
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
                } else {
                    // point to previous primary
                    IndexSplitResult {
                        inner: prev_primary,
                        location: *location,
                        path: vec![prev_loc],
                    }
                }
            } else {
                // wrap both primary and secondary side
                match self.child_pattern_offset_splits::<S>(
                        location.parent,
                        child_patterns,
                        offset,
                    ) {
                    Ok(result) => result,
                    Err(splits) =>
                        self.unperfect_splits(
                            location.parent,
                            splits,
                        )
                }
            })
        })
    }
    /// index inner half of pattern
    #[instrument(skip(self))]
    fn pattern_perfect_split<
        S: RelativeSide<D, Side>,
        P: IntoPattern,
        L: Borrow<ChildLocation> + Debug
    >(
        &'a mut self,
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
    fn child_pattern_offset_splits<
        S: RelativeSide<D, Side>,
    >(
        &'a mut self,
        parent: Child,
        child_patterns: ChildPatterns,
        offset: NonZeroUsize,
    ) -> Result<IndexSplitResult, Vec<(ChildLocation, Pattern, IndexSplitResult, Child)>> {
        let mut child_patterns = child_patterns.into_iter();
        let len = child_patterns.len();
        match child_patterns
            .try_fold(Vec::with_capacity(len), |mut acc, (pid, pattern)| {
                let (index, inner_offset) = Side::token_offset_split(pattern.borrow(), offset).unwrap();
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
                Err(
                    c.into_iter()
                        .map(|(pid, pattern, pos, offset)| {
                            let sub = *pattern.get(pos).unwrap();
                            // split index at pos with offset
                            let split = self.single_offset_split::<S>(
                                sub,
                                offset,
                            );

                            // index inner context
                            let IndexSplitResult {
                                inner: context,
                                ..
                            } = self.index_secondary_path::<S, _>(
                                split.path.clone().into_iter().chain(
                                    std::iter::once(split.location)
                                ),
                            ).unwrap();
                            (parent.to_child_location(pid, pos), pattern, split, context)
                        })
                        .collect()
                )
            },
        }
    }
    /// split parent at token offset from direction start
    #[instrument(skip(self, parent, offset))]
    pub fn single_offset_split<
        S: RelativeSide<D, Side>,
    >(
        &'a mut self,
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
        &'a mut self,
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
        &'a mut self,
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
            let mut fronts = HashSet::default();
            for (location, pattern, split, context) in splits {
                let pos = location.sub_index;
                let IndexSplitResult {
                    inner,
                    path: _,
                    location: _,
                } = split;
                let (back, front) = Side::context_inner_order(&context, &inner);
                // todo: order depends on D
                backs.insert([&D::back_context(pattern.borrow(), pos)[..], back].concat());
                fronts.insert([front, &D::front_context(pattern.borrow(), pos)[..]].concat());
            }
            
            //println!("{:#?}", backs);
            //println!("{:#?}", fronts);
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