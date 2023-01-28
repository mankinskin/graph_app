use crate::*;

type OppositeContextRange<D, Ty> =
    <<Ty as IndexSide<D>>::Opposite as IndexSide<D>>::ContextRange;

pub trait RelativeSide<D: IndexDirection, S: IndexSide<D>>: Sync + Send + Unpin {
    type Opposite: RelativeSide<D, S>;
    type Range: PatternRangeIndex + StartInclusive;
    fn is_context_side() -> bool;
    fn is_inner_side() -> bool {
        !Self::is_context_side()
    }
    fn exclusive_primary_index(index: usize) -> Option<usize>;
    fn exclusive_primary_location(location: ChildLocation) -> Option<ChildLocation> {
        Some(ChildLocation {
            sub_index: Self::exclusive_primary_index(location.sub_index)?,
            ..location
        })
    }
    fn exclusive_secondary_index(index: usize) -> Option<usize>;
    fn exclusive_secondary_location(location: ChildLocation) -> Option<ChildLocation> {
        Some(ChildLocation {
            sub_index: Self::exclusive_secondary_index(location.sub_index)?,
            ..location
        })
    }
    fn primary_range(index: usize) -> Self::Range;
    fn primary_indexed_pos(index: usize) -> usize;
    fn secondary_range(index: usize) -> <Self::Opposite as RelativeSide<D, S>>::Range {
        <Self::Opposite as RelativeSide<D, S>>::primary_range(index)
    }
    fn split_primary(pattern: &'_ impl IntoPattern, pos: usize) -> &'_ [Child] {
        &pattern.borrow()[Self::primary_range(pos)]
    }
    fn split_secondary(pattern: &'_ impl IntoPattern, pos: usize) -> &'_ [Child] {
        &pattern.borrow()[Self::secondary_range(pos)]
    }
    fn outer_inner_order(outer: Child, inner: Child) -> (Child, Child);
    fn index_inner_and_context<T: Tokenize>(indexer: &mut Indexer<T, D>, inner: Child, context: Child) -> Child;
    fn has_secondary_exclusive(pattern: &'_ impl IntoPattern, pos: usize) -> bool {
        Self::is_inner_side() && !Self::split_secondary(pattern, pos).is_empty()
            || Self::is_context_side() && Self::split_secondary(pattern, pos).len() > 1
    }
}

pub struct ContextSide;

impl<D: IndexDirection, S: IndexSide<D>> RelativeSide<D, S> for ContextSide {
    type Opposite = InnerSide;
    type Range = <S as IndexSide<D>>::ContextRange;
    fn is_context_side() -> bool {
        true
    }
    fn exclusive_primary_index(index: usize) -> Option<usize> {
        Some(index)
    }
    fn exclusive_primary_location(location: ChildLocation) -> Option<ChildLocation> {
        Some(location)
    }
    fn exclusive_secondary_index(index: usize) -> Option<usize> {
        <S as IndexSide<D>>::next_inner_index(index)
    }
    fn primary_range(index: usize) -> Self::Range {
        S::context_range(index)
    }
    fn primary_indexed_pos(index: usize) -> usize {
        S::inner_pos_after_context_indexed(index)
    }
    fn index_inner_and_context<T: Tokenize>(indexer: &mut Indexer<T, D>, inner: Child, context: Child) -> Child {
        let (back, front) = <Self as RelativeSide<D, S>>::outer_inner_order(context, inner);
        if let Ok((c, _)) = indexer.index_pattern([back, front]) {
            c
        } else {
            indexer.graph_mut().insert_pattern([back, front])
        }
        //indexer.graph_mut().insert_pattern([back, front])
    }
    fn outer_inner_order(outer: Child, inner: Child) -> (Child, Child) {
        let (back, front) = S::context_inner_order(&outer, &inner);
        (back[0], front[0])
    }
}
pub struct InnerSide;


impl<D: IndexDirection, S: IndexSide<D>> RelativeSide<D, S> for InnerSide {
    type Opposite = ContextSide;
    type Range = <S as IndexSide<D>>::InnerRange;
    fn is_context_side() -> bool {
        false
    }
    fn exclusive_primary_index(index: usize) -> Option<usize> {
        <S as IndexSide<D>>::next_inner_index(index)
    }
    fn exclusive_secondary_index(index: usize) -> Option<usize> {
        Some(index)
    }
    fn exclusive_secondary_location(location: ChildLocation) -> Option<ChildLocation> {
        Some(location)
    }
    fn primary_range(index: usize) -> Self::Range {
        S::inner_range(index)
    }
    fn primary_indexed_pos(index: usize) -> usize {
        index
    }
    fn index_inner_and_context<T: Tokenize>(indexer: &mut Indexer<T, D>, inner: Child, context: Child) -> Child {
        let (back, front) = <Self as RelativeSide<D, S>>::outer_inner_order(context, inner);
        //indexer.graph_mut().insert_pattern([back, front])
        match indexer.index_pattern([back, front]) {
            Ok((c, _)) => c,
            _ => indexer.graph_mut().insert_pattern([back, front]),
        }
    }
    fn outer_inner_order(outer: Child, inner: Child) -> (Child, Child) {
        let (back, front) = S::context_inner_order(&inner, &outer);
        (back[0], front[0])
    }
}
/// Side refers to border (front is indexing before front border, back is indexing after back border)
pub trait IndexSide<D: IndexDirection>: std::fmt::Debug + Sync + Send + Unpin + Clone + 'static {
    type Opposite: IndexSide<D>;
    type InnerRange: PatternRangeIndex + StartInclusive;
    type ContextRange: PatternRangeIndex + StartInclusive;
    type BottomUpPathIter: Iterator<Item=ChildLocation> + ExactSizeIterator + Send + Sync;
    fn next_outer_index(index: usize) -> Option<usize>;
    fn next_inner_index(index: usize) -> Option<usize>;
    fn inner_width_to_offset(child: &Child, width: usize) -> Option<NonZeroUsize>;
    /// returns inner, context
    fn back_front_order<A>(back: A, front: A) -> (A, A);
    /// returns back, front
    fn context_inner_order<
        'a,
        C: AsRef<[Child]>,
        I: AsRef<[Child]>
    >(context: &'a C, inner: &'a I) -> (&'a [Child], &'a [Child]);
    /// inner index and outer context
    fn concat_inner_and_context(
        inner: Child,
        context: impl IntoPattern,
    ) -> Pattern;
    /// inner index and inner context
    fn concat_inner_and_inner_context(
        inner: Child,
        inner_context: impl IntoPattern,
    ) -> Pattern {
        Self::Opposite::concat_inner_and_context(
            inner,
            inner_context
        )
    }
    fn inner_range(pos: usize) -> Self::InnerRange;
    fn is_split_at_border(pos: usize, pattern: impl IntoPattern) -> bool;
    fn inner_context_range(pos: usize) -> OppositeContextRange<D, Self> {
        Self::Opposite::context_range(pos)
    }
    fn context_range(pos: usize) -> Self::ContextRange;
    fn bottom_up_path_iter(
        path: impl IntoIterator<Item=impl Borrow<ChildLocation>>,
    ) -> Self::BottomUpPathIter;
    fn inner_pos_after_context_indexed(pos: usize) -> usize;
    fn limited_range(start: usize, end: usize) -> Range<usize>;
    fn range_front(range: &Range<usize>) -> usize;
    fn limited_inner_range(range: &Range<usize>) -> Range<usize>;
    fn max_range(pattern: impl IntoPattern, pos: usize) -> Range<usize>;
    #[track_caller]
    fn split_context(pattern: &'_ impl IntoPattern, pos: usize) -> &'_ [Child] {
        &pattern.borrow()[Self::context_range(pos)]
    }
    #[track_caller]
    fn split_inner(pattern: &'_ impl IntoPattern, pos: usize) -> &'_ [Child] {
        &pattern.borrow()[Self::inner_range(pos)]
    }
    fn sub_ranges(inner: &Range<usize>, limited: &Range<usize>) -> Range<usize>;
    fn token_offset_split(
        pattern: impl IntoPattern,
        offset: NonZeroUsize,
    ) -> Option<(usize, Option<NonZeroUsize>)>;
}

#[derive(Debug, Clone)]
pub struct IndexBack;
impl<D: IndexDirection> IndexSide<D> for IndexBack {
    type Opposite = IndexFront;
    //type Path = RolePath;
    type InnerRange = RangeFrom<usize>;
    type ContextRange = Range<usize>;
    type BottomUpPathIter = std::vec::IntoIter<ChildLocation>;
    fn inner_range(pos: usize) -> Self::InnerRange {
        pos..
    }
    fn next_outer_index(index: usize) -> Option<usize> {
        D::index_prev(index)
    }
    fn next_inner_index(index: usize) -> Option<usize> {
        D::index_next(index)
    }
    fn is_split_at_border(pos: usize, pattern: impl IntoPattern) -> bool {
        pos == D::head_index(pattern)
    }
    fn context_range(pos: usize) -> Self::ContextRange {
        0..pos
    }
    fn inner_pos_after_context_indexed(_pos: usize) -> usize {
        1
    }
    fn bottom_up_path_iter(
        path: impl IntoIterator<Item=impl Borrow<ChildLocation>>,
    ) -> Self::BottomUpPathIter {
        path.into_iter().map(|loc| loc.borrow().clone()).collect_vec().into_iter()
    }
    fn context_inner_order<
        'a,
        C: AsRef<[Child]>,
        I: AsRef<[Child]>
    >(context: &'a C, inner: &'a I) -> (&'a [Child], &'a [Child]) {
        (context.as_ref(), inner.as_ref())
    }
    fn concat_inner_and_context(
        inner: Child,
        context: impl IntoPattern,
    ) -> Pattern {
        D::context_then_inner(context, inner)
    }
    fn back_front_order<A>(back: A, front: A) -> (A, A) {
        (front, back)
    }
    fn inner_width_to_offset(child: &Child, width: usize) -> Option<NonZeroUsize> {
        // todo: changes with index direction
        if child.width() < width {
            assert!(child.width() >= width);
        }
        NonZeroUsize::new(child.width() - width)
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
        offset: NonZeroUsize,
    ) -> Option<(usize, Option<NonZeroUsize>)> {
        let mut offset = offset.get();
        pattern.into_iter()
            .enumerate()
            .find_map(|(i, c)|
                // returns current index when remaining offset is smaller than current child
                match c.width().cmp(&offset) {
                    Ordering::Less => {
                        offset -= c.width();
                        None
                    },
                    Ordering::Equal => {
                        offset = 0;
                        None
                    },
                    Ordering::Greater => Some((i, NonZeroUsize::new(offset))),
                }
            )
    }
}
#[derive(Debug, Clone)]
pub struct IndexFront;
impl<D: IndexDirection> IndexSide<D> for IndexFront {
    type Opposite = IndexBack;
    //type Path = RolePath;
    type InnerRange = Range<usize>;
    type ContextRange = RangeFrom<usize>;
    type BottomUpPathIter = std::iter::Rev<<Self::Opposite as IndexSide<D>>::BottomUpPathIter>;
    fn next_outer_index(index: usize) -> Option<usize> {
        D::index_next(index)
    }
    fn next_inner_index(index: usize) -> Option<usize> {
        D::index_prev(index)
    }
    fn inner_range(pos: usize) -> Self::InnerRange {
        0..<Self as IndexSide<D>>::next_outer_index(pos).unwrap()
    }
    fn context_range(pos: usize) -> Self::ContextRange {
        <Self as IndexSide<D>>::next_outer_index(pos).unwrap()..
    }
    fn bottom_up_path_iter(path: impl IntoIterator<Item=impl Borrow<ChildLocation>>) -> Self::BottomUpPathIter {
        path.into_iter().map(|loc| loc.borrow().clone()).collect_vec().into_iter().rev()
    }
    fn inner_pos_after_context_indexed(pos: usize) -> usize {
        pos
    }
    fn is_split_at_border(pos: usize, pattern: impl IntoPattern) -> bool {
        Some(pos) == <Self as IndexSide<D>>::next_outer_index(D::last_index(pattern))
    }
    fn context_inner_order<
        'a,
        C: AsRef<[Child]>,
        I: AsRef<[Child]>
    >(context: &'a C, inner: &'a I) -> (&'a [Child], &'a [Child]) {
        (inner.as_ref(), context.as_ref())
    }
    fn concat_inner_and_context(
        inner: Child,
        context: impl IntoPattern,
    ) -> Pattern {
        D::inner_then_context(inner, context)
    }
    fn back_front_order<A>(back: A, front: A) -> (A, A) {
        (back, front)
    }
    fn inner_width_to_offset(child: &Child, width: usize) -> Option<NonZeroUsize> {
        if width < child.width() {
            NonZeroUsize::new(width)
        } else {
            None
        }
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
        offset: NonZeroUsize,
    ) -> Option<(usize, Option<NonZeroUsize>)> {
        let mut offset = offset.get();
        pattern.into_iter()
            .enumerate()
            .find_map(|(i, c)|
                // returns current index when remaining offset does not exceed current child
                match c.width().cmp(&offset) {
                    Ordering::Less => {
                        offset -= c.width();
                        None
                    },
                    Ordering::Equal => {
                        offset = 0;
                        Some((i, NonZeroUsize::new(offset)))
                    },
                    Ordering::Greater => Some((i, NonZeroUsize::new(offset))),
                }
            )
    }
}

#[cfg(test)]
mod tests {
    use std::num::NonZeroUsize;
    use crate::{
        *,
        index::*,
    };


    
    fn token_offset_split() {
        let pattern = mock::pattern_from_widths([
            1,
            1,
            3,
            1,
            1,
        ]);
        let width = pattern_width(&pattern);
        assert_eq!(
            <IndexBack as IndexSide<Right>>::token_offset_split(
                pattern.borrow(),
                NonZeroUsize::new(2).unwrap(),
            ),
            Some((2, None)),
        );
        assert_eq!(
            <IndexFront as IndexSide<Right>>::token_offset_split(
                pattern.borrow(),
                NonZeroUsize::new(width-2).unwrap(),
            ),
            Some((2, None)),
        );
        assert_eq!(
            <IndexFront as IndexSide<Right>>::token_offset_split(
                pattern.borrow(),
                NonZeroUsize::new(width-4).unwrap(),
            ),
            Some((2, NonZeroUsize::new(1))),
        );
    }
}