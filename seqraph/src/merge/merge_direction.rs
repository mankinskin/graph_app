use crate::{
    split::*,
    vertex::*,
};

pub trait MergeDirection {
    type Opposite: MergeDirection;
    fn split_context_head(context: SplitSegment) -> Option<(Child, Pattern)>;
    fn split_inner_head(context: SplitSegment) -> (Child, Pattern) {
        Self::Opposite::split_context_head(context)
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
// context left, inner right
pub struct MergeLeft;
impl MergeDirection for MergeLeft {
    type Opposite = MergeRight;
    fn split_context_head(context: SplitSegment) -> Option<(Child, Pattern)> {
        match context {
            SplitSegment::Pattern(mut p) => {
                let last = p.pop();
                last.map(|last| (last, p))
            },
            SplitSegment::Child(c) => Some((c, vec![])),
        }
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
pub struct MergeRight;
impl MergeDirection for MergeRight {
    type Opposite = MergeLeft;
    fn split_context_head(context: SplitSegment) -> Option<(Child, Pattern)> {
        match context {
            SplitSegment::Pattern(p) => {
                let mut p = p.into_iter();
                let first = p.next();
                first.map(|last| (last, p.collect()))
            },
            SplitSegment::Child(c) => Some((c, vec![])),
        }
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
