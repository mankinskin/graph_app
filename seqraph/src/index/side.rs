use std::num::NonZeroUsize;

use super::*;
use crate::*;
type OppositeContextRange<D, Ty> =
    <<Ty as IndexSide<D>>::Opposite as IndexSide<D>>::ContextRange;

/// Side refers to border (front is indexing before front border, back is indexing after back border)
pub(crate) trait IndexSide<D: IndexDirection> {
    type Opposite: IndexSide<D>;
    type Path: DirectedBorderPath<D>;
    type InnerRange: PatternRangeIndex + StartInclusive;
    type ContextRange: PatternRangeIndex + StartInclusive;
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
    fn split_at_border(pos: usize, pattern: impl IntoPattern) -> bool;
    fn inner_context_range(pos: usize) -> OppositeContextRange<D, Self> {
        Self::Opposite::context_range(pos)
    }
    fn context_range(pos: usize) -> Self::ContextRange;
    fn limited_range(start: usize, end: usize) -> Range<usize>;
    fn range_front(range: &Range<usize>) -> usize;
    fn limited_inner_range(range: &Range<usize>) -> Range<usize>;
    fn max_range(pattern: impl IntoPattern, pos: usize) -> Range<usize>;
    fn split_context(pattern: &'_ impl IntoPattern, pos: usize) -> &'_ [Child];
    fn sub_ranges(inner: &Range<usize>, limited: &Range<usize>) -> Range<usize>;
    fn token_offset_split(
        pattern: impl IntoPattern,
        offset: NonZeroUsize,
    ) -> Option<(usize, Option<NonZeroUsize>)>;
}

pub(crate) struct IndexBack;
impl<D: IndexDirection> IndexSide<D> for IndexBack {
    type Opposite = IndexFront;
    type Path = StartPath;
    type InnerRange = RangeFrom<usize>;
    type ContextRange = Range<usize>;
    fn inner_range(pos: usize) -> Self::InnerRange {
        pos..
    }
    fn split_at_border(pos: usize, pattern: impl IntoPattern) -> bool {
        pos == D::head_index(pattern)
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
    fn concat_inner_and_context(
        inner: Child,
        context: impl IntoPattern,
    ) -> Pattern {
        D::concat_context_and_inner(context, inner)
    }
    fn back_front_order<A>(back: A, front: A) -> (A, A) {
        (front, back)
    }
    #[track_caller]
    fn split_context(pattern: &'_ impl IntoPattern, pos: usize) -> &'_ [Child] {
        &pattern.borrow()[..pos]
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
pub(crate) struct IndexFront;
impl<D: IndexDirection> IndexSide<D> for IndexFront {
    type Opposite = IndexBack;
    type Path = EndPath;
    type InnerRange = Range<usize>;
    type ContextRange = RangeFrom<usize>;
    fn inner_range(pos: usize) -> Self::InnerRange {
        0..D::index_next(pos).unwrap()
    }
    fn context_range(pos: usize) -> Self::ContextRange {
        D::index_next(pos).unwrap()..
    }
    fn split_at_border(pos: usize, pattern: impl IntoPattern) -> bool {
        Some(pos) == D::index_next(D::last_index(pattern))
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
    fn concat_inner_and_context(
        inner: Child,
        context: impl IntoPattern,
    ) -> Pattern {
        D::concat_inner_and_context(inner, context)
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
                        None
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


    #[test]
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
            Some((3, None)),
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