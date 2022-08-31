use std::num::NonZeroUsize;
use crate::*;
use super::*;
type HashSet<T> = DeterministicHashSet<T>;
type HashMap<K, V> = DeterministicHashMap<K, V>;

pub(crate) trait SideIndexable<'a: 'g, 'g, T: Tokenize, D: IndexDirection, Side: IndexSide<D>>: Indexable<'a, 'g, T, D> {
    fn entry_perfect_split(
        &'a mut self,
        entry: ChildLocation,
    ) -> Option<IndexSplitResult> {
        let mut graph = self.graph_mut();
        let pattern = graph.expect_pattern_at(&entry);       
        SideIndexable::<_, D, Side>::pattern_perfect_split(
            &mut *graph,
            pattern,
            entry,
        )
    }
    /// split pattern at location
    fn pattern_perfect_split(
        &'a mut self,
        pattern: impl IntoPattern,
        location: ChildLocation,
    ) -> Option<IndexSplitResult> {
        if Side::split_at_border(
            location.sub_index,
            pattern.borrow(),
        ) {
            return None;
        }
        let range = Side::inner_range(location.sub_index);
        let inner = &pattern.borrow()[range.clone()];
        if inner.is_empty() {
            assert!(!inner.is_empty());
        }
        let inner = if inner.len() == 1 {
            *inner.iter().next().unwrap()
        } else {
            let mut graph = self.graph_mut();
            let inner = graph.insert_pattern(inner).unwrap();
            graph.replace_in_pattern(&location, range.clone(), [inner]);
            inner
        };
        Some(IndexSplitResult {
            location: location.to_child_location(range.start()),
            path: vec![],
            inner,
        })
    }
    /// split parent at token offset from direction start
    fn path_segment_split(
        &'a mut self,
        prev: Option<IndexSplitResult>,
        seg: ChildLocation,
    ) -> Option<IndexSplitResult> {
        let mut graph = self.graph_mut();
        if let Some(mut prev) = prev {
            // index lower context
            let (split_context, split_location) = SideIndexable::<_, _, Side>::context_path(
                &mut *graph,
                prev.location,
                prev.path.clone(),
                prev.inner,
            );
            let inner_context_range = Side::inner_context_range(seg.sub_index);
            let inner_context = graph.expect_child_pattern_range(
                seg,
                inner_context_range,
            );
            if inner_context.is_empty() {
                Some(IndexSplitResult {
                    location: seg,
                    path: vec![
                        split_location
                    ],
                    inner: prev.inner,
                })
            } else {
                let inner_pat = Side::concat_inner_and_inner_context(
                    prev.inner,
                    inner_context.borrow(),
                );
                let offset = Side::inner_width_to_offset(
                    &seg.parent,
                    pattern::pattern_width(inner_pat),
                ).unwrap();
                let child_patterns = graph.expect_child_patterns_of(seg.parent)
                    .into_iter()
                    .filter(|(id, _)| **id != seg.pattern_id)
                    .map(|(id, p)| (*id, p.clone()))
                    .collect::<HashMap<_, _>>();
                let _s = format!("{:#?}", child_patterns);
                let mut splits =
                    SideIndexable::< _, _, Side>::child_pattern_offset_splits(
                        &mut* graph,
                        seg.parent,
                        child_patterns.clone(),
                        offset,
                    ).expect_err("Other pattern with split at same offset!");
                prev.location = split_location;
                splits.push((seg, graph.expect_pattern_at(seg), prev, split_context));
                Some(SideIndexable::< _, _, Side>::unperfect_splits(
                    &mut *graph,
                    seg.parent,
                    splits,
                ))
            }
        } else {
            SideIndexable::< _, _, Side>::entry_perfect_split(
                &mut* graph,
                seg,
            )
        }
    }
    fn single_entry_split(
        &'a mut self,
        entry: ChildLocation,
        path: ChildPath,
    ) -> Option<IndexSplitResult> {
        let mut graph = self.graph_mut();
        let prev = SideIndexable::< _, _, Side>::single_path_split(
            &mut *graph,
            path,
        );
        SideIndexable::< _, _, Side>::path_segment_split(
            &mut *graph,
            prev,
            entry,
        )
    }
    fn single_path_split(
        &'a mut self,
        path: ChildPath,
    ) -> Option<IndexSplitResult> {
        let mut graph = self.graph_mut();
        let mut prev: Option<IndexSplitResult> = None;
        let mut path = Side::bottom_up_path_iter(path);
        while let Some(seg) = path.next() {
            prev = SideIndexable::< _, _, Side>::path_segment_split(
                &mut *graph,
                prev,
                seg,
            )
        }
        prev
    }
    fn child_pattern_offset_splits(
        &'a mut self,
        parent: Child,
        child_patterns: ChildPatterns,
        offset: NonZeroUsize,
    ) -> Result<Option<IndexSplitResult>, Vec<(ChildLocation, Pattern, IndexSplitResult, Child)>> {
        let len = child_patterns.len();
        assert!(len > 0);
        let mut graph = self.graph_mut();
        match child_patterns.into_iter()
            .try_fold(Vec::with_capacity(len), |mut acc, (pid, pattern)| {
                let (index, inner_offset) = Side::token_offset_split(pattern.borrow(), offset).unwrap();
                if let Some(inner_offset) = inner_offset {
                    acc.push((pid, pattern.into_pattern(), index, inner_offset));
                    ControlFlow::Continue(acc)
                } else {
                    ControlFlow::Break((pattern.into_pattern(), pid, index))
                }
            })
        {
            ControlFlow::Break((pattern, pid, pos)) =>
                Ok(
                    SideIndexable::< _, _, Side>::pattern_perfect_split(
                        &mut *graph,
                        pattern,
                        ChildLocation::new(parent, pid, pos),
                    ),
                ),
            ControlFlow::Continue(c) => {
                Err(
                    c.into_iter()
                        .map(|(pid, pattern, pos, offset)| {
                            let sub = *pattern.get(pos).unwrap();
                            // split index at pos with offset
                            let split = SideIndexable::<_, _, Side>::single_offset_split(
                                &mut *graph,
                                sub,
                                offset,
                            ).unwrap();

                            // index inner context
                            let (context, _) = SideIndexable::<_, _, Side>::context_path(
                                &mut *graph,
                                split.location,
                                split.path.clone(),
                                split.inner,
                            );
                            (parent.to_child_location(pid, pos), pattern, split, context)
                        })
                        .collect()
                )
            },
        }
    }
    /// split parent at token offset from direction start
    fn single_offset_split(
        &'a mut self,
        parent: Child,
        offset: NonZeroUsize,
    ) -> Option<IndexSplitResult> {
        if offset.get() >= parent.width() {
            assert!(offset.get() < parent.width());
        }
        let mut graph = self.graph_mut();
        let child_patterns = graph.expect_child_patterns_of(&parent).clone();
        // find perfect split in parent
        match SideIndexable::<_, _, Side>::child_pattern_offset_splits(
            &mut *graph,
            parent,
            child_patterns,
            offset
        ) {
            Ok(split) => split,
            Err(splits) =>
                Some(
                    SideIndexable::< _, _, Side>::unperfect_splits(
                        &mut *graph,
                        parent,
                        splits,
                    )
                ),
        }
    }
    fn child_pattern_unperfect_split(
        &'a mut self,
        location: ChildLocation,
        split: IndexSplitResult,
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
        if let Some(parent_inner) = graph.index_range_in(
                location,
                Side::inner_context_range(pos)
            ).ok()
        {
            let full_inner = graph.index_pattern(
                Side::concat_inner_and_context(inner, parent_inner)
            );
            // index for inner half including split
            let wrapper = graph.index_range_in(
                location,
                Side::inner_range(pos),
            ).unwrap();
            let wrapper_pid = graph.add_pattern_with_update(
                wrapper,
                Side::concat_inner_and_context(inner, parent_inner)
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
    fn unperfect_splits(
        &'a mut self,
        parent: Child,
        splits: Vec<(ChildLocation, Pattern, IndexSplitResult, Child)>,
    ) -> IndexSplitResult {
        let mut graph = self.graph_mut();
        if splits.len() == 1 {
            let (location, _, split, _context) = splits.into_iter().next().unwrap();
            SideIndexable::<_, _, Side>::child_pattern_unperfect_split(
                &mut *graph,
                location,
                split,
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
            let (back, front) = (
                graph.index_patterns(backs),
                graph.index_patterns(fronts),
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
    /// * `location` - Points to child to index the context of
    fn context_path_segment(
        &'a mut self,
        location: ChildLocation
    ) -> (Child, ChildLocation) {
        let mut graph = self.graph_mut();
        let pattern = graph.expect_pattern_at(&location);
        let context = Side::split_context(&pattern, location.sub_index);
        if context.len() < 2 {
            if context.is_empty() {
                assert!(!context.is_empty());
            }
            (*context.into_iter().next().unwrap(), location)
        } else {
            let c = graph.index_pattern(context);
            let range = Side::context_range(location.sub_index);
            graph.replace_in_pattern(location, range, c);
            (c, location.to_child_location(Side::inner_pos_after_context_indexed(location.sub_index)))
        }
    }
    /// * `entry` - Points to child to index the context of
    /// * `context_path` - List of locations pointing into entry to build the nested context structure
    fn context_path(
        &'a mut self,
        entry: ChildLocation,
        mut context_path: Vec<ChildLocation>,
        inner: Child,
    ) -> (Child, ChildLocation) {
        let mut graph = self.graph_mut();
        let mut acc: Option<Child> = None;
        while let Some(location) = context_path.pop() {
            let (context, _inner_location) = SideIndexable::<_, _, Side>::context_path_segment(&mut *graph, location);
            if let Some(acc) = &mut acc {
                let (back, front) = Side::context_inner_order(&context, &acc);
                let context = graph.index_pattern([back[0], front[0]]);
                graph.add_pattern_with_update(location, Side::concat_inner_and_context(inner, context));
                *acc = context;
            } else {
                acc = Some(context);
            }
        }
        let (context, inner_location)
            = SideIndexable::<_, _, Side>::context_path_segment(&mut *graph, entry);
        if let Some(acc) = acc {
            let (back, front) = Side::context_inner_order(&context, &acc);
            let context = graph.index_pattern([back[0], front[0]]);
            let pid = graph.add_pattern_with_update(entry, Side::concat_inner_and_context(inner, context));
            let (sub_index, _) = Side::back_front_order(0, 1);
            (context, ChildLocation {
                parent: inner_location.parent,
                pattern_id: pid,
                sub_index,
            })
        } else {
            (context, inner_location)
        }
    }
}
impl<
    'a: 'g,
    'g,
    T: Tokenize,
    D: IndexDirection,
    Trav: Indexable<'a, 'g, T, D>,
    S: IndexSide<D>,
> SideIndexable<'a, 'g, T, D, S> for Trav {}