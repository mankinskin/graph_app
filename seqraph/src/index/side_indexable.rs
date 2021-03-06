use std::{num::NonZeroUsize, collections::HashSet};

use crate::*;
use super::*;

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
        assert!(!inner.is_empty());
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
    fn single_offset_split(
        &'a mut self,
        parent: Child,
        offset: NonZeroUsize,
    ) -> Option<IndexSplitResult> {
        if offset.get() >= parent.width() {
            assert!(offset.get() < parent.width());
        }
        let mut graph = self.graph_mut();
        let child_patterns = graph.expect_children_of(parent).clone();
        let len = child_patterns.len();
        // find perfect split in parent
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
                SideIndexable::< _, _, Side>::pattern_perfect_split(
                    &mut *graph,
                    pattern,
                    ChildLocation::new(parent, pid, pos),
                ),
            ControlFlow::Continue(positions) =>
                Some(if positions.len() == 1 {
                    let (pid, pattern, pos, offset) = positions.into_iter().next().unwrap();
                    SideIndexable::<_, _, Side>::child_pattern_unperfect_split(
                        &mut *graph,
                        pattern.borrow(),
                        parent.to_child_location(pid, pos),
                        offset,
                    )
                } else {
                    SideIndexable::< _, _, Side>::unperfect_splits(
                        &mut *graph,
                        parent,
                        positions,
                    )
                }),
        }
    }
    fn child_pattern_unperfect_split(
        &'a mut self,
        pattern: impl IntoPattern,
        location: impl IntoChildLocation,
        offset: NonZeroUsize,
    ) -> IndexSplitResult {
        let mut graph = self.graph_mut();
        let location = location.into_child_location();
        let pos = location.sub_index;

        let sub = *pattern.borrow().get(pos).unwrap();
        // split index at pos with offset
        let IndexSplitResult {
            inner,
            path,
            location: split_location,
        } = SideIndexable::<_, _, Side>::single_offset_split(
            &mut *graph,
            sub,
            offset,
        ).unwrap();

        // index inner context
        let _split_context = SideIndexable::<_, _, Side>::context_path(
            &mut *graph,
            split_location,
            path,
        );
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
        positions: Vec<(PatternId, Pattern, usize, NonZeroUsize)>,
    ) -> IndexSplitResult {
        // todo: fix resulting locations, fix D order
        let mut graph = self.graph_mut();
        let (backs, fronts) = positions.into_iter()
            .map(|(_, pattern, pos, offset)| {
                let IndexSplitResult {
                    inner,
                    path,
                    location,
                } = SideIndexable::<_, _, Side>::single_offset_split(
                    &mut *graph,
                    *pattern.get(pos).unwrap(),
                    offset,
                ).unwrap();
                let context = SideIndexable::<_, _, Side>::context_path(&mut *graph, location, path);
                let (back, front) = Side::context_inner_order(&context, &inner);
                (
                    // todo: order depends on D
                    [&D::back_context(pattern.borrow(), pos)[..], back].concat(),
                    [front, &D::front_context(pattern.borrow(), pos)[..]].concat(),
                )
            }).unzip::<_, _, HashSet<_>, HashSet<_>>();
        
        //println!("{:#?}", backs);
        //println!("{:#?}", fronts);
        let (back, front) = (
            graph.index_patterns(backs),
            graph.index_patterns(fronts),
        );
        let pid = graph.add_pattern_with_update(parent, [back, front]);
        let (inner, _) = Side::back_front_order(back, front);
        let (pos, _) = Side::back_front_order(0, 1);
        let location = ChildLocation::new(parent, pid, pos);
        IndexSplitResult {
            location,
            path: vec![],
            inner,
        }
    }
    /// * `location` - Points to child to index the context of
    fn context_path_segment(
        &'a mut self,
        location: ChildLocation
    ) -> Child {
        let mut graph = self.graph_mut();
        let pattern = graph.expect_pattern_at(&location);
        let context = Side::split_context(&pattern, location.sub_index);
        if context.len() < 2 {
            if context.is_empty() {
                assert!(!context.is_empty());
            }
            *context.into_iter().next().unwrap()
        } else {
            let context = graph.index_pattern(context);
            graph.replace_in_pattern(location, Side::context_range(location.sub_index), context);
            context
        }
    }
    /// * `entry` - Points to child to index the context of
    /// * `context_path` - List of locations pointing into entry to build the nested context structure
    fn context_path(
        &'a mut self,
        entry: ChildLocation,
        mut context_path: Vec<ChildLocation>
    ) -> Child {
        let mut graph = self.graph_mut();
        let mut acc: Option<Child> = None;
        while let Some(location) = context_path.pop() {
            let context = SideIndexable::<_, _, Side>::context_path_segment(&mut *graph, location);
            if let Some(acc) = &mut acc {
                let (back, front) = Side::context_inner_order(&context, &acc);
                *acc = graph.index_pattern([back[0], front[0]]);
            } else {
                acc = Some(context);
            }
        }
        let context = SideIndexable::<_, _, Side>::context_path_segment(&mut *graph, entry);
        if let Some(acc) = acc {
            let (back, front) = Side::context_inner_order(&context, &acc);
            graph.index_pattern([back[0], front[0]])
        } else {
            context
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