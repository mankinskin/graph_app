use crate::{
    split::*,
    vertex::*,
};
pub use crate::direction::*;

pub trait MergeDirection: Direction {
    type Opposite: MergeDirection;
    fn split_context_head(context: impl Merge) -> Option<(Child, Pattern)>;
    fn split_inner_head(context: impl Merge) -> (Child, Pattern) {
        <Self as MergeDirection>::Opposite::split_context_head(context)
            .expect("Empty inner pattern!")
    }
    fn concat_inner_and_context(
        inner_context: Pattern,
        inner: Pattern,
        outer_context: Pattern,
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
impl Merge for SplitSegment {
    fn split_front(self) -> Option<(Child, Pattern)> {
        match self {
            SplitSegment::Pattern(p) => p.split_front(),
            SplitSegment::Child(c) => c.split_front(),
        }
    }
    fn split_back(self) -> Option<(Child, Pattern)> {
        match self {
            SplitSegment::Pattern(p) => p.split_back(),
            SplitSegment::Child(c) => c.split_back(),
        }
    }
}
impl MergeDirection for Left {
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
        inner_context: Pattern,
        inner: Pattern,
        outer_context: Pattern,
    ) -> Pattern {
        [outer_context, inner, inner_context].concat()
    }
}
// context right, inner left
impl MergeDirection for Right {
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
        inner_context: Pattern,
        inner: Pattern,
        outer_context: Pattern,
    ) -> Pattern {
        [inner_context, inner, outer_context].concat()
    }
}
