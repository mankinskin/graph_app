use super::*;
use crate::*;

/// Side refers to border (front is indexing before front border, back is indexing after back border)
pub(crate) trait IndexSide<D: IndexDirection> {
    type Path: DirectedBorderPath<D>;
    type InnerRange: PatternRangeIndex + StartInclusive;
    type ContextRange: PatternRangeIndex + StartInclusive;
    fn width_offset(parent: &Child, width: usize) -> usize;
    /// returns inner, context
    fn back_front_order<A>(back: A, front: A) -> (A, A);
    /// returns back, front
    fn context_inner_order<
        'a,
        C: AsRef<[Child]>,
        I: AsRef<[Child]>
    >(context: &'a C, inner: &'a I) -> (&'a [Child], &'a [Child]);
    fn inner_range(pos: usize) -> Self::InnerRange;
    fn context_range(pos: usize) -> Self::ContextRange;
    fn limited_range(start: usize, end: usize) -> Range<usize>;
    fn range_front(range: &Range<usize>) -> usize;
    fn limited_inner_range(range: &Range<usize>) -> Range<usize>;
    fn max_range(pattern: impl IntoPattern, pos: usize) -> Range<usize>;
    fn split_context(pattern: &'_ impl IntoPattern, pos: usize) -> &'_ [Child];
    fn sub_ranges(inner: &Range<usize>, limited: &Range<usize>) -> Range<usize>;
    fn token_offset_split(
        pattern: impl IntoPattern,
        offset: usize,
    ) -> Option<(usize, usize)>;
}
pub(crate) struct IndexBack;
impl<D: IndexDirection> IndexSide<D> for IndexBack {
    type Path = StartPath;
    type InnerRange = RangeFrom<usize>;
    type ContextRange = Range<usize>;
    fn inner_range(pos: usize) -> Self::InnerRange {
        pos..
    }
    fn context_range(pos: usize) -> Self::ContextRange {
        0..pos
    }
    fn context_inner_order<
        'a,
        C: AsRef<[Child]>,
        I: AsRef<[Child]>
    >(context: &'a C, inner: &'a I) -> (&'a [Child], &'a [Child]) {
        (context.as_ref(), inner.as_ref())
    }
    fn back_front_order<A>(back: A, front: A) -> (A, A) {
        (front, back)
    }
    #[track_caller]
    fn split_context(pattern: &'_ impl IntoPattern, pos: usize) -> &'_ [Child] {
        &pattern.borrow()[..pos]
    }
    fn width_offset(parent: &Child, width: usize) -> usize {
        // todo: changes with index direction
        parent.width() - width
    }
    fn limited_range(start: usize, end: usize) -> Range<usize> {
        start..end
    }
    fn range_front(range: &Range<usize>) -> usize {
        range.start()
    }
    fn limited_inner_range(range: &Range<usize>) -> Range<usize> {
        D::index_next(range.start()).unwrap()..range.end()
    }
    fn max_range(pattern: impl IntoPattern, pos: usize) -> Range<usize> {
        pos..pattern.borrow().len()
    }
    fn sub_ranges(inner: &Range<usize>, limited: &Range<usize>) -> Range<usize> {
        limited.start()-inner.start()..limited.end()-limited.start()
    }
    fn token_offset_split(
        pattern: impl IntoPattern,
        offset: usize,
    ) -> Option<(usize, usize)> {
        D::pattern_offset_context_split(pattern, offset)
    }
}
pub(crate) struct IndexFront;
impl<D: IndexDirection> IndexSide<D> for IndexFront {
    type Path = EndPath;
    type InnerRange = RangeInclusive<usize>;
    type ContextRange = RangeFrom<usize>;
    fn inner_range(pos: usize) -> Self::InnerRange {
        0..=pos
    }
    fn context_range(pos: usize) -> Self::ContextRange {
        D::index_next(pos).unwrap()..
    }
    #[track_caller]
    fn split_context(pattern: &'_ impl IntoPattern, pos: usize) -> &'_ [Child] {
        &pattern.borrow()[D::index_next(pos).unwrap()..]
    }
    fn context_inner_order<
        'a,
        C: AsRef<[Child]>,
        I: AsRef<[Child]>
    >(context: &'a C, inner: &'a I) -> (&'a [Child], &'a [Child]) {
        (inner.as_ref(), context.as_ref())
    }
    fn back_front_order<A>(back: A, front: A) -> (A, A) {
        (back, front)
    }
    fn width_offset(_parent: &Child, width: usize) -> usize {
        width
    }
    fn limited_range(start: usize, end: usize) -> Range<usize> {
        start..D::index_next(end).unwrap()
    }
    fn range_front(range: &Range<usize>) -> usize {
        D::index_prev(range.end()).unwrap()
    }
    fn limited_inner_range(range: &Range<usize>) -> Range<usize> {
        range.start()..D::index_prev(range.end()).unwrap()
    }
    fn sub_ranges(_inner: &Range<usize>, limited: &Range<usize>) -> Range<usize> {
        0..limited.end()-limited.start()
    }
    fn max_range(_pattern: impl IntoPattern, pos: usize) -> Range<usize> {
        0..D::index_next(pos).unwrap()
    }
    fn token_offset_split(
        pattern: impl IntoPattern,
        offset: usize,
    ) -> Option<(usize, usize)> {
        D::pattern_offset_inner_split(pattern, offset)
    }
}

pub(crate) trait SideIndexable<'a: 'g, 'g, T: Tokenize, D: IndexDirection, Side: IndexSide<D>>: Indexable<'a, 'g, T, D> {
    /// todo: a little bit dirty because width always points to a perfect split
    /// if the graph path segment it comes from is a leaf node
    fn index_entry_split(
        &'a mut self,
        entry: ChildLocation,
        width: usize,
    ) -> IndexSplitResult {
        let offset = Side::width_offset(&entry.parent, width);
        self.index_offset_split(entry.parent, offset)
    }
    fn index_perfect_split(
        &'a mut self,
        entry: ChildLocation
    ) -> IndexSplitResult {
        let mut graph = self.graph_mut();
        let pattern = graph.expect_pattern_at(&entry);
        SideIndexable::<_, _, Side>::pattern_index_perfect_split(&mut *graph, pattern, entry)
    }
    fn pattern_index_perfect_split(
        &'a mut self,
        pattern: Pattern,
        entry: ChildLocation,
    ) -> IndexSplitResult {
        Self::pattern_index_perfect_split_range(self, pattern, entry, Side::inner_range(entry.sub_index))
    }
    fn index_context_path_segment(
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
    #[named]
    fn index_context_path(
        &'a mut self,
        entry: ChildLocation,
        mut context_path: Vec<ChildLocation>
    ) -> Child {
        trace!(function_name!());
        let mut graph = self.graph_mut();
        let mut acc: Option<Child> = None;
        while let Some(location) = context_path.pop() {
            let context = SideIndexable::<_, _, Side>::index_context_path_segment(&mut *graph, location);
            if let Some(acc) = &mut acc {
                let (back, front) = Side::context_inner_order(&context, &acc);
                *acc = graph.index_pattern([back[0], front[0]]);
            } else {
                acc = Some(context);
            }
        }
        let context = SideIndexable::<_, _, Side>::index_context_path_segment(&mut *graph, entry);
        if let Some(acc) = acc {
            let (back, front) = Side::context_inner_order(&context, &acc);
            graph.index_pattern([back[0], front[0]])
        } else {
            context
        }
    }
    #[named]
    fn pattern_range_unperfect_split(
        &'a mut self,
        pattern: Pattern,
        location: impl IntoPatternLocation,
        offset: usize,
        range: Range<usize>,
    ) -> IndexSplitResult {
        trace!(function_name!());
        let (_index, offset) = Side::token_offset_split(&pattern[range.clone()], offset).unwrap();
        self.pattern_index_unperfect_split(pattern, location, offset, range)
    }
    #[named]
    fn pattern_index_unperfect_split(
        &'a mut self,
        pattern: Pattern,
        location: impl IntoPatternLocation,
        offset: usize,
        range: Range<usize>,
    ) -> IndexSplitResult {
        trace!(function_name!());
        let mut graph = self.graph_mut();
        let location@PatternLocation {
            parent,
            pattern_id: pid,
        } = location.into_pattern_location();
        // range_front of max_range & limited_range with pos must equal pos
        let pos = Side::range_front(&range);
        let IndexSplitResult {
            inner: split_inner,
            context,
            location: split_location,
            // split index at pos with offset
        } = SideIndexable::<_, _, Side>::index_offset_split(&mut *graph, *pattern.get(pos).unwrap(), offset);

        let split_context = SideIndexable::<_, _, Side>::index_context_path(&mut *graph, split_location, context);

        let (split_back, split_front) = Side::context_inner_order(&split_context, &split_inner);
        // includes split index
        let mut old = pattern[range.clone()].to_vec();
        // range from indexing start (back) until split index (front)
        let inner_range = Side::limited_inner_range(&range);
        let old_inner = graph.index_pattern(&pattern[inner_range.clone()]);
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
            context: vec![ChildLocation::new(inner, new_pid, 1)],
            inner: new_inner,
        }
    }
    #[named]
    fn index_unperfect_splits(
        &'a mut self,
        parent: Child,
        positions: Vec<(PatternId, Pattern, usize, usize)>,
    ) -> IndexSplitResult {
        trace!(function_name!());
        // todo: fix resulting locations, fix D order
        let mut graph = self.graph_mut();
        if positions.len() == 1 {
            let (pid, pattern, pos, offset) = positions.into_iter().next().unwrap();
            let range = Side::max_range(pattern.borrow(), pos);
            SideIndexable::<_, _, Side>::pattern_index_unperfect_split(&mut *graph, pattern, parent.to_pattern_location(pid), offset, range)
        } else {
            let (backs, fronts) = positions.into_iter()
                .map(|(_, pattern, pos, offset)| {
                    let IndexSplitResult {
                        inner,
                        context,
                        location,
                    } = SideIndexable::<_, _, Side>::index_offset_split(&mut *graph, *pattern.get(pos).unwrap(), offset);
                    let context = SideIndexable::<_, _, Side>::index_context_path(&mut *graph, location, context);
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
                context: vec![],
                inner,
            }
        }
    }
    #[named]
    #[instrument(skip(self))]
    fn index_offset_split(
        &'a mut self,
        parent: Child,
        offset: usize,
    ) -> IndexSplitResult {
        //assert!(offset != 0);
        trace!(function_name!());
        let mut graph = self.graph_mut();
        let child_patterns = graph.expect_children_of(parent).clone();
        let len = child_patterns.len();
        let perfect = child_patterns.into_iter()
            .try_fold(Vec::with_capacity(len), |mut acc, (pid, pattern)| {
                let (index, inner_offset) = Side::token_offset_split(pattern.borrow(), offset).unwrap();
                if inner_offset == 0 {
                    ControlFlow::Break((pattern.into_pattern(), pid, index))
                } else {
                    acc.push((pid, pattern.into_pattern(), index, inner_offset));
                    ControlFlow::Continue(acc)
                }
            });
        match perfect {
            ControlFlow::Break((pattern, pid, pos)) =>
                SideIndexable::< _, _, Side>::pattern_index_perfect_split(&mut *graph, pattern, ChildLocation::new(parent, pid, pos)),
            ControlFlow::Continue(positions) =>
                SideIndexable::< _, _, Side>::index_unperfect_splits(&mut *graph, parent, positions),
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