use std::ops::{Range, RangeInclusive};

use crate::*;

pub trait IndexDirection: MatchDirection + Clone {
    type Opposite: IndexDirection;
    fn split_context_head(context: impl Merge) -> Option<(Child, Pattern)>;
    fn split_last(context: impl Merge) -> Option<(Pattern, Child)> {
        <Self as IndexDirection>::Opposite::split_context_head(context)
            .map(|(c, rem)| (rem, c))
    }
    fn split_inner_head(context: impl Merge) -> (Child, Pattern) {
        <Self as IndexDirection>::Opposite::split_context_head(context)
            .expect("Empty inner pattern!")
    }
    // first inner, then context
    fn inner_then_context(
        inner: Child,
        context: impl IntoPattern,
    ) -> Pattern;
    // first context, then inner
    fn context_then_inner(
        context: impl IntoPattern,
        inner: Child,
    ) -> Pattern {
        <Self as IndexDirection>::Opposite::inner_then_context(
            inner,
            context
        )
    }
    fn merge_order(
        inner: Child,
        head: Child,
    ) -> (Child, Child);
    fn inner_context_range(
        back: usize,
        front: usize,
    ) -> Range<usize>;
    fn wrapper_range(
        back: usize,
        front: usize,
    ) -> RangeInclusive<usize>;
    fn concat_context_inner_context(
        head_context: Child,
        inner: impl IntoPattern,
        last_context: Child,
    ) -> Pattern;
}
pub trait Merge {
    fn split_front(self) -> Option<(Child, Pattern)>;
    fn split_back(self) -> Option<(Child, Pattern)>;
}
impl Merge for Child {
    fn split_front(self) -> Option<(Child, Pattern)> {
        Some((self, vec![]))
    }
    fn split_back(self) -> Option<(Child, Pattern)> {
        Some((self, vec![]))
    }
}
impl Merge for Pattern {
    fn split_front(self) -> Option<(Child, Pattern)> {
        let mut p = self.into_iter();
        let first = p.next();
        first.map(|last| (last, p.collect()))
    }
    fn split_back(mut self) -> Option<(Child, Pattern)> {
        let last = self.pop();
        last.map(|last| (last, self))
    }
}
impl IndexDirection for Left {
    type Opposite = Right;
    fn split_context_head(context: impl Merge) -> Option<(Child, Pattern)> {
        context.split_back()
    }
    fn merge_order(
        inner: Child,
        head: Child,
    ) -> (Child, Child) {
        (head, inner)
    }
    fn inner_then_context(
        inner: Child,
        context: impl IntoPattern,
    ) -> Pattern {
        context.borrow().to_owned().into_iter().chain(inner).collect()
    }
    fn inner_context_range(
        back: usize,
        front: usize,
    ) -> Range<usize> {
        Self::index_prev(front).unwrap()..back
    }
    fn wrapper_range(
        back: usize,
        front: usize,
    ) -> RangeInclusive<usize> {
        front..=back
    }
    fn concat_context_inner_context(
        head_context: Child,
        inner: impl IntoPattern,
        last_context: Child,
    ) -> Pattern {
        std::iter::once(last_context)
            .chain(inner.borrow().to_owned())
            .chain(std::iter::once(head_context))
            .collect()
    }
}
impl IndexDirection for Right {
    type Opposite = Left;
    fn split_context_head(context: impl Merge) -> Option<(Child, Pattern)> {
        context.split_front()
    }
    fn merge_order(
        inner: Child,
        head: Child,
    ) -> (Child, Child) {
        (inner, head)
    }
    fn inner_then_context(
        inner: Child,
        context: impl IntoPattern,
    ) -> Pattern {
        std::iter::once(inner).chain(context.borrow().to_owned()).collect()
    }
    fn concat_context_inner_context(
        head_context: Child,
        inner: impl IntoPattern,
        last_context: Child,
    ) -> Pattern {
        std::iter::once(head_context)
            .chain(inner.borrow().to_owned())
            .chain(std::iter::once(last_context))
            .collect()
    }
    fn inner_context_range(
        back: usize,
        front: usize,
    ) -> Range<usize> {
        Self::index_next(back).unwrap()..front
    }
    fn wrapper_range(
        back: usize,
        front: usize,
    ) -> RangeInclusive<usize> {
        back..=front
    }
}