//type OppositeContextRange<D, Ty> =
//    <<Ty as IndexSide<D>>::Opposite as IndexSide<D>>::ContextRange;

use std::{
    cmp::Ordering,
    num::NonZeroUsize,
};

use crate::vertex::{
    pattern::IntoPattern,
    wide::Wide,
};

/// Side refers to border (front is indexing before front border, back is indexing after back border)
pub trait IndexSide: std::fmt::Debug + Sync + Send + Unpin + Clone + 'static {
    //type Opposite: IndexSide<D>;
    //type InnerRange: PatternRangeIndex + StartInclusive;
    //type ContextRange: PatternRangeIndex + StartInclusive;
    //type BottomUpPathIter: Iterator<Item=ChildLocation> + ExactSizeIterator + Send + Sync;
    //fn next_outer_index(index: usize) -> Option<usize>;
    //fn next_inner_index(index: usize) -> Option<usize>;
    //fn inner_width_to_offset(child: &Child, width: usize) -> Option<NonZeroUsize>;
    ///// returns inner, context
    //fn back_front_order<A>(back: A, front: A) -> (A, A);
    ///// returns back, front
    //fn context_inner_order<
    //    'a,
    //    C: AsRef<[Child]>,
    //    I: AsRef<[Child]>
    //>(context: &'a C, inner: &'a I) -> (&'a [Child], &'a [Child]);
    ///// inner index and outer context
    //fn concat_inner_and_context(
    //    inner: Child,
    //    context: impl IntoPattern,
    //) -> Pattern;
    ///// inner index and inner context
    //fn concat_inner_and_inner_context(
    //    inner: Child,
    //    inner_context: impl IntoPattern,
    //) -> Pattern {
    //    Self::Opposite::concat_inner_and_context(
    //        inner,
    //        inner_context
    //    )
    //}
    //fn inner_range(pos: usize) -> Self::InnerRange;
    //fn is_split_at_border(pos: usize, pattern: impl IntoPattern) -> bool;
    //fn inner_context_range(pos: usize) -> OppositeContextRange<D, Self> {
    //    Self::Opposite::context_range(pos)
    //}
    //fn context_range(pos: usize) -> Self::ContextRange;
    //fn bottom_up_path_iter(
    //    path: impl IntoIterator<Item=impl Borrow<ChildLocation>>,
    //) -> Self::BottomUpPathIter;
    //fn inner_pos_after_context_indexed(pos: usize) -> usize;
    //fn limited_range(start: usize, end: usize) -> Range<usize>;
    //fn range_front(range: &Range<usize>) -> usize;
    //fn limited_inner_range(range: &Range<usize>) -> Range<usize>;
    //fn max_range(pattern: impl IntoPattern, pos: usize) -> Range<usize>;
    //#[track_caller]
    //fn split_context(pattern: &'_ impl IntoPattern, pos: usize) -> &'_ [Child] {
    //    &pattern.borrow()[Self::context_range(pos)]
    //}
    //#[track_caller]
    //fn split_inner(pattern: &'_ impl IntoPattern, pos: usize) -> &'_ [Child] {
    //    &pattern.borrow()[Self::inner_range(pos)]
    //}
    //fn sub_ranges(inner: &Range<usize>, limited: &Range<usize>) -> Range<usize>;
    fn token_offset_split(
        pattern: impl IntoPattern,
        offset: NonZeroUsize,
    ) -> Option<(usize, Option<NonZeroUsize>)>;
}

#[derive(Debug, Clone)]
pub struct IndexBack;
impl IndexSide for IndexBack {
    //type Opposite = IndexFront;
    ////type Path = RolePath;
    //type InnerRange = RangeFrom<usize>;
    //type ContextRange = Range<usize>;
    //type BottomUpPathIter = std::vec::IntoIter<ChildLocation>;
    //fn inner_range(pos: usize) -> Self::InnerRange {
    //    pos..
    //}
    //fn next_outer_index(index: usize) -> Option<usize> {
    //    D::index_prev(index)
    //}
    //fn next_inner_index(index: usize) -> Option<usize> {
    //    D::index_next(index)
    //}
    //fn is_split_at_border(pos: usize, pattern: impl IntoPattern) -> bool {
    //    pos == D::head_index(pattern)
    //}
    //fn context_range(pos: usize) -> Self::ContextRange {
    //    0..pos
    //}
    //fn inner_pos_after_context_indexed(_pos: usize) -> usize {
    //    1
    //}
    //fn bottom_up_path_iter(
    //    path: impl IntoIterator<Item=impl Borrow<ChildLocation>>,
    //) -> Self::BottomUpPathIter {
    //    path.into_iter().map(|loc| loc.borrow().clone()).collect_vec().into_iter()
    //}
    //fn context_inner_order<
    //    'a,
    //    C: AsRef<[Child]>,
    //    I: AsRef<[Child]>
    //>(context: &'a C, inner: &'a I) -> (&'a [Child], &'a [Child]) {
    //    (context.as_ref(), inner.as_ref())
    //}
    //fn concat_inner_and_context(
    //    inner: Child,
    //    context: impl IntoPattern,
    //) -> Pattern {
    //    D::context_then_inner(context, inner)
    //}
    //fn back_front_order<A>(back: A, front: A) -> (A, A) {
    //    (front, back)
    //}
    //fn inner_width_to_offset(child: &Child, width: usize) -> Option<NonZeroUsize> {
    //    // todo: changes with index direction
    //    if child.width() < width {
    //        assert!(child.width() >= width);
    //    }
    //    NonZeroUsize::new(child.width() - width)
    //}
    //fn limited_range(start: usize, end: usize) -> Range<usize> {
    //    start..end
    //}
    //fn range_front(range: &Range<usize>) -> usize {
    //    range.start()
    //}
    //fn limited_inner_range(range: &Range<usize>) -> Range<usize> {
    //    D::index_next(range.start()).unwrap()..range.end()
    //}
    //fn max_range(pattern: impl IntoPattern, pos: usize) -> Range<usize> {
    //    pos..pattern.borrow().len()
    //}
    //fn sub_ranges(inner: &Range<usize>, limited: &Range<usize>) -> Range<usize> {
    //    limited.start()-inner.start()..limited.end()-limited.start()
    //}
    fn token_offset_split(
        pattern: impl IntoPattern,
        offset: NonZeroUsize,
    ) -> Option<(usize, Option<NonZeroUsize>)> {
        let mut offset = offset.get();
        pattern.into_iter().enumerate().find_map(|(i, c)|
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
                })
    }
}
#[derive(Debug, Clone)]
pub struct IndexFront;
impl IndexSide for IndexFront {
    //type Opposite = IndexBack;
    ////type Path = RolePath;
    //type InnerRange = Range<usize>;
    //type ContextRange = RangeFrom<usize>;
    //type BottomUpPathIter = std::iter::Rev<<Self::Opposite as IndexSide<D>>::BottomUpPathIter>;
    //fn next_outer_index(index: usize) -> Option<usize> {
    //    D::index_next(index)
    //}
    //fn next_inner_index(index: usize) -> Option<usize> {
    //    D::index_prev(index)
    //}
    //fn inner_range(pos: usize) -> Self::InnerRange {
    //    0..<Self as IndexSide<D>>::next_outer_index(pos).unwrap()
    //}
    //fn context_range(pos: usize) -> Self::ContextRange {
    //    <Self as IndexSide<D>>::next_outer_index(pos).unwrap()..
    //}
    //fn bottom_up_path_iter(path: impl IntoIterator<Item=impl Borrow<ChildLocation>>) -> Self::BottomUpPathIter {
    //    path.into_iter().map(|loc| loc.borrow().clone()).collect_vec().into_iter().rev()
    //}
    //fn inner_pos_after_context_indexed(pos: usize) -> usize {
    //    pos
    //}
    //fn is_split_at_border(pos: usize, pattern: impl IntoPattern) -> bool {
    //    Some(pos) == <Self as IndexSide<D>>::next_outer_index(D::last_index(pattern))
    //}
    //fn context_inner_order<
    //    'a,
    //    C: AsRef<[Child]>,
    //    I: AsRef<[Child]>
    //>(context: &'a C, inner: &'a I) -> (&'a [Child], &'a [Child]) {
    //    (inner.as_ref(), context.as_ref())
    //}
    //fn concat_inner_and_context(
    //    inner: Child,
    //    context: impl IntoPattern,
    //) -> Pattern {
    //    D::inner_then_context(inner, context)
    //}
    //fn back_front_order<A>(back: A, front: A) -> (A, A) {
    //    (back, front)
    //}
    //fn inner_width_to_offset(child: &Child, width: usize) -> Option<NonZeroUsize> {
    //    if width < child.width() {
    //        NonZeroUsize::new(width)
    //    } else {
    //        None
    //    }
    //}
    //fn limited_range(start: usize, end: usize) -> Range<usize> {
    //    start..D::index_next(end).unwrap()
    //}
    //fn range_front(range: &Range<usize>) -> usize {
    //    D::index_prev(range.end()).unwrap()
    //}
    //fn limited_inner_range(range: &Range<usize>) -> Range<usize> {
    //    range.start()..D::index_prev(range.end()).unwrap()
    //}
    //fn sub_ranges(_inner: &Range<usize>, limited: &Range<usize>) -> Range<usize> {
    //    0..limited.end()-limited.start()
    //}
    //fn max_range(_pattern: impl IntoPattern, pos: usize) -> Range<usize> {
    //    0..D::index_next(pos).unwrap()
    //}
    fn token_offset_split(
        pattern: impl IntoPattern,
        offset: NonZeroUsize,
    ) -> Option<(usize, Option<NonZeroUsize>)> {
        let mut offset = offset.get();
        pattern.into_iter().enumerate().find_map(|(i, c)|
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
                })
    }
}

#[cfg(test)]
mod tests {
    use crate::vertex::pattern::pattern_width;
    use std::{
        borrow::Borrow,
        num::NonZeroUsize,
    };

    #[test]
    fn token_offset_split() {
        let pattern = mock::pattern_from_widths([1, 1, 3, 1, 1]);
        let width = pattern_width(&pattern);
        assert_eq!(
            IndexBack::token_offset_split(
                pattern.borrow() as &[Child],
                NonZeroUsize::new(2).unwrap(),
            ),
            Some((2, None)),
        );
        assert_eq!(
            IndexFront::token_offset_split(
                pattern.borrow() as &[Child],
                NonZeroUsize::new(width - 2).unwrap(),
            ),
            Some((2, None)),
        );
        assert_eq!(
            IndexFront::token_offset_split(
                pattern.borrow() as &[Child],
                NonZeroUsize::new(width - 4).unwrap(),
            ),
            Some((2, NonZeroUsize::new(1))),
        );
    }
}
