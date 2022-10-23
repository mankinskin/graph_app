use std::num::NonZeroUsize;

use crate::*;
use super::*;

type HashSet<T> = DeterministicHashSet<T>;
type HashMap<K, V> = DeterministicHashMap<K, V>;

pub(crate) trait IndexSplit<'a: 'g, 'g, T: Tokenize, D: IndexDirection, Side: IndexSide<D>>: Indexing<'a, 'g, T, D> {
    fn entry_perfect_split(
        &'a mut self,
        entry: ChildLocation,
    ) -> Option<IndexSplitResult> {
        let mut graph = self.graph_mut();
        let pattern = graph.expect_pattern_at(&entry);       
        IndexSplit::<_, D, Side>::pattern_perfect_split(
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
            let (split_context, split_location) = IndexContext::<_, _, Side>::context_entry_path(
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
                // split other patterns at offset
                let other_patterns = graph.expect_child_patterns_of(seg.parent)
                    .into_iter()
                    .filter(|(id, _)| **id != seg.pattern_id)
                    .map(|(id, p)| (*id, p.clone()))
                    .collect::<HashMap<_, _>>();
                let _s = format!("{:#?}", other_patterns);
                let mut splits =
                    IndexSplit::< _, _, Side>::child_pattern_offset_splits(
                        &mut* graph,
                        seg.parent,
                        other_patterns,
                        offset,
                    ).expect_err("Other pattern with split at same offset!");

                prev.location = split_location;
                splits.push((seg, graph.expect_pattern_at(seg), prev, split_context));

                Some(IndexSplit::< _, _, Side>::unperfect_splits(
                    &mut *graph,
                    seg.parent,
                    splits,
                ))
            }
        } else {
            IndexSplit::< _, _, Side>::entry_perfect_split(
                &mut* graph,
                seg,
            )
        }
    }
    fn single_entry_split(
        &'a mut self,
        entry: ChildLocation,
        path: impl IntoIterator<Item=impl Borrow<ChildLocation>>,
    ) -> Option<IndexSplitResult> {
        let path = path.into_iter().collect_vec();
        let mut graph = self.graph_mut();
        let prev = IndexSplit::< _, _, Side>::single_path_split(
            &mut *graph,
            path,
        );
        IndexSplit::< _, _, Side>::path_segment_split(
            &mut *graph,
            prev,
            entry,
        )
    }
    fn single_path_split(
        &'a mut self,
        path: impl IntoIterator<Item=impl Borrow<ChildLocation>>,
    ) -> Option<IndexSplitResult> {
        let path = path.into_iter().collect_vec();
        let mut graph = self.graph_mut();
        let mut prev: Option<IndexSplitResult> = None;
        let mut path = Side::bottom_up_path_iter(path);
        while let Some(seg) = path.next() {
            prev = IndexSplit::< _, _, Side>::path_segment_split(
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
    ) -> Result<IndexSplitResult, Vec<(ChildLocation, Pattern, IndexSplitResult, Child)>> {
        let len = child_patterns.len();
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
                    IndexSplit::< _, _, Side>::pattern_perfect_split(
                        &mut *graph,
                        pattern,
                        ChildLocation::new(parent, pid, pos),
                    ).expect("Offset non-zero!"),
                ),
            ControlFlow::Continue(c) => {
                Err(
                    c.into_iter()
                        .map(|(pid, pattern, pos, offset)| {
                            let sub = *pattern.get(pos).unwrap();
                            // split index at pos with offset
                            let split = IndexSplit::<_, _, Side>::single_offset_split(
                                &mut *graph,
                                sub,
                                offset,
                            );

                            // index inner context
                            let (context, _) = IndexContext::<_, _, Side>::context_entry_path(
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
    ) -> IndexSplitResult {
        if offset.get() >= parent.width() {
            assert!(offset.get() < parent.width());
        }
        let mut graph = self.graph_mut();
        let child_patterns = graph.expect_child_patterns_of(&parent).clone();
        // find perfect split in parent
        match IndexSplit::<_, _, Side>::child_pattern_offset_splits(
            &mut *graph,
            parent,
            child_patterns,
            offset
        ) {
            Ok(split) => split,
            Err(splits) =>
                IndexSplit::< _, _, Side>::unperfect_splits(
                    &mut *graph,
                    parent,
                    splits,
                ),
        }
    }
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
        if let Some(parent_inner) = graph.index_range_in(
                location,
                Side::inner_context_range(pos)
            ).ok()
        {
            // split_inner + split inner context
            let full_inner = graph.index_pattern(
                // context on opposite side than usual (inner side)
                <Side as IndexSide<_>>::Opposite::concat_inner_and_context(inner, parent_inner),
            );
            // ||    |     ||      |
            //       ^^^^^^^^^^^^^^
            // index for inner half including split
            if let Ok(wrapper) = graph.index_range_in(
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
    fn unperfect_splits(
        &'a mut self,
        parent: Child,
        splits: Vec<(ChildLocation, Pattern, IndexSplitResult, Child)>,
    ) -> IndexSplitResult {
        let mut graph = self.graph_mut();
        if splits.len() == 1 {
            let (location, _, split, context) = splits.into_iter().next().unwrap();
            IndexSplit::<_, _, Side>::entry_unperfect_split(
                &mut *graph,
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
}
impl<
    'a: 'g,
    'g,
    T: Tokenize,
    D: IndexDirection,
    Trav: Indexing<'a, 'g, T, D>,
    S: IndexSide<D>,
> IndexSplit<'a, 'g, T, D, S> for Trav {}