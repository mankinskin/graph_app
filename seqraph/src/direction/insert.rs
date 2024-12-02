use std::ops::{
    Range,
    RangeInclusive,
};

use crate::{
    direction::{
        Left,
        Right,
    },
    graph::direction::{
        r#match::MatchDirection,
        merge::Merge,
    },
};
use crate::graph::vertex::{
    child::Child,
    pattern::{
        IntoPattern,
        Pattern,
    },
};

pub trait InsertDirection: MatchDirection + Clone + PartialEq + Eq {
    type Opposite: InsertDirection;
    fn split_context_head(context: impl Merge) -> Option<(Child, Pattern)>;
    fn split_last(context: impl Merge) -> Option<(Pattern, Child)> {
        <Self as InsertDirection>::Opposite::split_context_head(context).map(|(c, rem)| (rem, c))
    }
    fn split_inner_head(context: impl Merge) -> (Child, Pattern) {
        <Self as InsertDirection>::Opposite::split_context_head(context)
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
        <Self as InsertDirection>::Opposite::inner_then_context(inner, context)
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



impl InsertDirection for Left {
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
        context
            .borrow()
            .iter()
            .copied()
            .chain(inner)
            .collect()
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

impl InsertDirection for Right {
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
        std::iter::once(inner)
            .chain(context.borrow().to_owned())
            .collect()
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
