use context_search::{
    direction::{
        Direction,
        Left,
        Right,
    },
    graph::vertex::{
        child::Child,
        pattern::Pattern,
    },
};

pub trait InsertDirection: Direction + Clone + PartialEq + Eq
{
    fn context_then_inner(
        context: Pattern,
        inner: Child,
    ) -> Pattern
    where
        Self::Opposite: InsertDirection,
    {
        <<Self as Direction>::Opposite as InsertDirection>::inner_then_context(
            inner, context,
        )
    }

    fn inner_then_context(
        inner: Child,
        context: Pattern,
    ) -> Pattern;

    //fn split_context_head(context: impl Merge) -> Option<(Child, Pattern)>;
    //fn split_last(context: impl Merge) -> Option<(Pattern, Child)> {
    //    <Self as InsertDirection>::Opposite::split_context_head(context).map(|(c, rem)| (rem, c))
    //}
    //fn split_inner_head(context: impl Merge) -> (Child, Pattern) {
    //    <Self as InsertDirection>::Opposite::split_context_head(context)
    //        .expect("Empty inner pattern!")
    //}
    //// first inner, then context
    //// first context, then inner
    //fn merge_order(
    //    inner: Child,
    //    head: Child,
    //) -> (Child, Child);
    //fn inner_context_range(
    //    back: usize,
    //    front: usize,
    //) -> Range<usize>;
    //fn wrapper_range(
    //    back: usize,
    //    front: usize,
    //) -> RangeInclusive<usize>;
    //fn concat_context_inner_context(
    //    head_context: Child,
    //    inner: impl IntoPattern,
    //    last_context: Child,
    //) -> Pattern;
}

impl InsertDirection for Left
{
    fn inner_then_context(
        inner: Child,
        context: Pattern,
    ) -> Pattern
    {
        context.iter().copied().chain(inner).collect()
    }

    //fn split_context_head(context: impl Merge) -> Option<(Child, Pattern)> {
    //    context.split_back()
    //}
    //fn merge_order(
    //    inner: Child,
    //    head: Child,
    //) -> (Child, Child) {
    //    (head, inner)
    //}
    //fn inner_context_range(
    //    back: usize,
    //    front: usize,
    //) -> Range<usize> {
    //    Self::index_prev(front).unwrap()..back
    //}
    //fn wrapper_range(
    //    back: usize,
    //    front: usize,
    //) -> RangeInclusive<usize> {
    //    front..=back
    //}
    //fn concat_context_inner_context(
    //    head_context: Child,
    //    inner: impl IntoPattern,
    //    last_context: Child,
    //) -> Pattern {
    //    std::iter::once(last_context)
    //        .chain(inner.borrow().to_owned())
    //        .chain(std::iter::once(head_context))
    //        .collect()
    //}
}

impl InsertDirection for Right
{
    fn inner_then_context(
        inner: Child,
        context: Pattern,
    ) -> Pattern
    {
        std::iter::once(inner).chain(context.to_owned()).collect()
    }

    //fn split_context_head(context: impl Merge) -> Option<(Child, Pattern)> {
    //    context.split_front()
    //}
    //fn merge_order(
    //    inner: Child,
    //    head: Child,
    //) -> (Child, Child) {
    //    (inner, head)
    //}
    //fn concat_context_inner_context(
    //    head_context: Child,
    //    inner: impl IntoPattern,
    //    last_context: Child,
    //) -> Pattern {
    //    std::iter::once(head_context)
    //        .chain(inner.borrow().to_owned())
    //        .chain(std::iter::once(last_context))
    //        .collect()
    //}
    //fn inner_context_range(
    //    back: usize,
    //    front: usize,
    //) -> Range<usize> {
    //    Self::index_next(back).unwrap()..front
    //}
    //fn wrapper_range(
    //    back: usize,
    //    front: usize,
    //) -> RangeInclusive<usize> {
    //    back..=front
    //}
}
