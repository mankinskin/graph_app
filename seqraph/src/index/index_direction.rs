use crate::{
    index::*,
};
pub use crate::direction::*;

pub trait IndexDirection: MatchDirection {
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
    fn concat_inner_and_context(
        inner: Child,
        overlap: Option<Child>,
        outer_context: Pattern,
    ) -> Pattern;
    fn concat_inner_and_outer(
        inner: Pattern,
        outer: Pattern,
    ) -> Pattern;
    fn merge_order(
        inner: Child,
        head: Child,
    ) -> (Child, Child);
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
    fn concat_inner_and_context(
        inner: Child,
        overlap: Option<Child>,
        outer: Pattern,
    ) -> Pattern {
        outer.into_iter().chain(overlap).chain(inner).collect()
    }
    fn concat_inner_and_outer(
        inner: Pattern,
        outer: Pattern,
    ) -> Pattern {
        [outer, inner].concat()
    }
}
// context right, inner left
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
    fn concat_inner_and_context(
        inner: Child,
        overlap: Option<Child>,
        outer: Pattern,
    ) -> Pattern {
        std::iter::once(inner).chain(overlap).chain(outer).collect()
    }
    fn concat_inner_and_outer(
        inner: Pattern,
        outer: Pattern,
    ) -> Pattern {
        [inner, outer].concat()
    }
}