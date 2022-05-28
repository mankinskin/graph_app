use std::num::NonZeroUsize;

use crate::*;
use super::*;

//pub(crate) enum SplitRange {
//    InnerBothPerfect {
//        start: NonZeroUsize,
//        end: NonZeroUsize,
//    },
//    PerfectPostfix {
//        pos: NonZeroUsize,
//    },
//    PerfectPrefix {
//        pos: NonZeroUsize,
//    },
//    PostfixOff {
//        pos: usize,
//        offset: NonZeroUsize,
//    },
//    PrefixOff {
//        pos: usize,
//        offset: NonZeroUsize,
//    },
//    InnerBackOff {
//        start: usize,
//        offset: NonZeroUsize,
//        end: NonZeroUsize,
//    },
//    InnerFrontOff {
//        start: NonZeroUsize,
//        end: NonZeroUsize,
//        offset: NonZeroUsize,
//    },
//    InnerBothOff {
//        start: usize,
//        start_offset: NonZeroUsize,
//        // todo: how to handle direction in pattern indices
//        end: NonZeroUsize,
//        end_offset: NonZeroUsize,
//    },
//}
//impl SplitRange {
//    pub fn new<D: IndexDirection>(start: StartPath, end: EndPath, pattern: impl IntoPattern) -> Self {
//        match (
//            start.is_perfect(),
//            DirectedBorderPath::<D>::is_at_pattern_border(&start, pattern.borrow()),
//            end.is_perfect(),
//            DirectedBorderPath::<D>::is_at_pattern_border(&end, pattern.borrow()),
//        ) {
//            //   start         end
//            // perf comp    perf   comp
//            (true, true, true, true) =>
//                panic!("IndexingPath references complete index!"),
//            (true, _, true, true) =>
//                Self::PerfectPostfix {
//                    pos: NonZeroUsize::new(start.get_entry_pos()).unwrap(),
//                },
//            (true, true, true, _) =>
//                Self::PerfectPrefix {
//                    pos: NonZeroUsize::new(end.get_exit_pos()).unwrap(),
//                },
//            (false, _, true, true) =>
//                Self::PostfixOff {
//                    pos: start.get_entry_pos(),
//                    offset: NonZeroUsize::new(pattern[start.get_entry_pos()].width - start.width()).unwrap(),
//                },
//            (true, true, false, _) =>
//                Self::PrefixOff {
//                    pos: start.get_entry_pos(),
//                    offset: NonZeroUsize::new(end.width()).unwrap(),
//                },
//            (true, _, true, _) => {
//                let entry = start.get_entry_pos();
//                let exit = end.get_exit_pos();
//                assert!(entry != exit);
//                Self::InnerBothPerfect {
//                    start: NonZeroUsize::new(entry).unwrap(),
//                    end: NonZeroUsize::new(exit).unwrap(),
//                }
//            },
//            (false, _, true, false) =>
//                Self::InnerBackOff {
//                    start: start.get_entry_pos(),
//                    offset: NonZeroUsize::new(pattern.borrow()[start.get_entry_pos()].width - start.width()).unwrap(),
//                    end: NonZeroUsize::new(end.get_exit_pos()).unwrap(),
//                },
//            (true, false, false, _) =>
//                Self::InnerFrontOff {
//                    start: NonZeroUsize::new(start.get_entry_pos()).unwrap(),
//                    end: NonZeroUsize::new(end.get_exit_pos()).unwrap(),
//                    offset: NonZeroUsize::new(end.width()).unwrap(),
//                },
//            (false, _, false, _) =>
//                Self::InnerBothOff {
//                    start: start.get_entry_pos(),
//                    start_offset: NonZeroUsize::new(pattern[start.get_entry_pos()].width - start.width()).unwrap(),
//                    end: NonZeroUsize::new(end.get_exit_pos()).unwrap(),
//                    end_offset: NonZeroUsize::new(end.width()).unwrap(),
//                },
//        }
//    }
//}

pub(crate) trait SideIndexable<'a: 'g, 'g, T: Tokenize, D: IndexDirection, Side: IndexSide<D>>: Indexable<'a, 'g, T, D> {
    fn entry_split(
        &'a mut self,
        entry: ChildLocation,
        inner_width: usize,
    ) -> IndexSplitResult {
        let mut graph = self.graph_mut();
        let pattern = graph.expect_pattern_at(&entry);
        SideIndexable::<_, D, Side>::pattern_entry_split(
            &mut *graph,
            pattern.borrow(),
            entry,
            inner_width,
        )
    }
    fn pattern_entry_split(
        &'a mut self,
        pattern: impl IntoPattern,
        entry: ChildLocation,
        inner_width: usize,
    ) -> IndexSplitResult {
        let target = pattern.borrow()[entry.sub_index];
        match Side::inner_width_to_offset(&target, inner_width) {
            Some(offset) =>
                self.single_offset_split(
                    target,
                    offset,
                ),
            None => self.pattern_perfect_split(
                pattern,
                entry,
            ),
        }
    }
    fn pattern_perfect_split(
        &'a mut self,
        pattern: impl IntoPattern,
        entry: ChildLocation,
    ) -> IndexSplitResult {
        let range = Side::inner_range(entry.sub_index);
        self.pattern_range_perfect_split(pattern, entry, range)
    }
    fn single_offset_split(
        &'a mut self,
        parent: Child,
        offset: NonZeroUsize,
    ) -> IndexSplitResult {
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
                if positions.len() == 1 {
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
                },
        }
    }
    fn child_pattern_unperfect_split(
        &'a mut self,
        pattern: impl IntoPattern,
        location: impl IntoChildLocation,
        offset: NonZeroUsize,
    ) -> IndexSplitResult {
        let mut graph = self.graph_mut();
        let location@ChildLocation {
            parent,
            pattern_id: pid,
            sub_index: pos,
        } = location.into_child_location();
        let sub = *pattern.borrow().get(pos).unwrap();

        // split index at pos with offset
        let IndexSplitResult {
            inner: split_inner,
            path,
            location: split_location,
        } = SideIndexable::<_, _, Side>::single_offset_split(&mut *graph, sub, offset);

        let split_context = SideIndexable::<_, _, Side>::context_path(
            &mut *graph,
            split_location,
            path,
        );

        let (split_back, split_front) = Side::context_inner_order(&split_context, &split_inner);
        // includes split index
        let mut old = pattern.borrow()[range.clone()].to_vec();
        // range from indexing start (back) until split index (front)
        let inner_range = Side::limited_inner_range(&range);
        let old_inner = graph.index_pattern(&pattern.borrow()[inner_range.clone()]);
        let old_inner_range = Side::sub_ranges(&range, &inner_range);
        replace_in_pattern(&mut old, old_inner_range, old_inner);

        let (inner_back, inner_front) = Side::context_inner_order(&split_back, &old_inner);
        let new_inner = graph.index_pattern([inner_back[0], inner_front[0]]);
        let (back, front) = Side::context_inner_order(&split_front, &new_inner);
        let new = [back[0], front[0]];

        let (inner, ids) = graph.index_patterns_with_ids([&new, &old[..]]);
        let new_pid = ids[0];
        graph.replace_in_pattern(location, range, inner);
        let location = ChildLocation::new(parent, pid, pos);
        IndexSplitResult {
            location,
            path: vec![ChildLocation::new(inner, new_pid, 1)],
            inner: new_inner,
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
                );
                let context = SideIndexable::<_, _, Side>::context_path(&mut *graph, location, path);
                let (back, front) = Side::context_inner_order(&context, &inner);
                (
                    // todo: order depends on D
                    [&D::back_context(pattern.borrow(), pos)[..], back].concat(),
                    [front, &D::front_context(pattern.borrow(), pos)[..]].concat(),
                )
            }).unzip::<_, _, Vec<_>, Vec<_>>();
        let (back, front) = (
            graph.index_patterns(backs),
            graph.index_patterns(fronts),
        );
        let pid = graph.add_pattern_with_update(parent, [back, front]);
        let (inner, _) = Side::back_front_order(back, front);
        let location = ChildLocation::new(parent, pid, 1);
        IndexSplitResult {
            location,
            path: vec![],
            inner,
        }
        
    }
    fn context_path_segment(
        &'a mut self,
        location: ChildLocation
    ) -> Child {
        let mut graph = self.graph_mut();
        let pattern = graph.expect_pattern_at(&location);
        let context = Side::split_context(&pattern, location.sub_index);
        let context = graph.index_pattern(context);
        // todo: skip if not needed
        graph.replace_in_pattern(location, Side::context_range(location.sub_index), context);
        context
    }
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