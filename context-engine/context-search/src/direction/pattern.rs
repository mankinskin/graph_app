use crate::graph::vertex::pattern::Pattern;

use super::{
    Direction,
    Left,
    Right,
};

pub trait PatternDirection: Direction {
    //fn pattern_tail<T: ToChild>(pattern: &'_ [T]) -> &'_ [T];
    //fn pattern_head<T: ToChild>(pattern: &'_ [T]) -> Option<&'_ T>;
    fn head_index(pattern: &Pattern) -> usize;
    fn last_index(pattern: &Pattern) -> usize
    where
        <Self as Direction>::Opposite: PatternDirection,
    {
        Self::Opposite::head_index(pattern)
    }
    fn index_next(index: usize) -> Option<usize>;
    fn index_prev(index: usize) -> Option<usize>;
    fn pattern_index_next(
        pattern: &Pattern,
        index: usize,
    ) -> Option<usize> {
        Self::index_next(index).and_then(|i| (i < pattern.len()).then_some(i))
    }
    fn pattern_index_prev(
        pattern: &Pattern,
        index: usize,
    ) -> Option<usize> {
        Self::index_prev(index).and_then(|i| (i < pattern.len()).then_some(i))
    }
}

impl PatternDirection for Right {
    fn head_index(_pattern: &Pattern) -> usize {
        0
    }
    fn index_next(index: usize) -> Option<usize> {
        index.checked_add(1)
    }
    fn index_prev(index: usize) -> Option<usize> {
        index.checked_sub(1)
    }
}

impl PatternDirection for Left {
    fn head_index(pattern: &Pattern) -> usize {
        pattern.len() - 1
    }
    fn index_next(index: usize) -> Option<usize> {
        index.checked_sub(1)
    }
    fn index_prev(index: usize) -> Option<usize> {
        index.checked_add(1)
    }
}

//pub trait MatchDirection: Direction {
//    type PostfixRange<T>: PatternRangeIndex<T>;
//    /// get the parent where vertex is at the relevant position
//    fn get_match_parent_to<G: GraphKind>(
//        graph: &Hypergraph<G>,
//        vertex: &VertexData,
//        sup: impl HasVertexIndex,
//    ) -> Result<PatternIndex, ErrorReason>;
//    fn skip_equal_indices<'a, I: HasVertexIndex, J: HasVertexIndex>(
//        a: impl DoubleEndedIterator<Item = &'a I>,
//        b: impl DoubleEndedIterator<Item = &'a J>,
//    ) -> Option<(usize, EitherOrBoth<&'a I, &'a J>)>;
//    /// get remaining pattern in matching direction including index
//    fn merge_remainder_with_context<A: IntoPattern, B: IntoPattern>(
//        rem: A,
//        context: B,
//    ) -> Pattern;
//    fn tail_index<T: ToChild>(
//        pattern: &'_ [T],
//        tail: &'_ [T],
//    ) -> usize;
//    /// filter pattern indices of parent relation by child patterns and matching direction
//    fn filter_parent_pattern_indices(
//        parent: &Parent,
//        child_patterns: &HashMap<PatternId, Pattern>,
//    ) -> HashSet<PatternIndex>;
//    fn split_head_tail<T: ToChild + Clone>(pattern: &'_ [T]) -> Option<(T, &'_ [T])> {
//        Self::pattern_head(pattern).map(|head| (head.clone(), Self::pattern_tail(pattern)))
//    }
//    fn front_context_range<T>(index: usize) -> Self::PostfixRange<T>;
//    /// get remaining pattern in matching direction excluding index
//    fn front_context<T: ToChild + Clone>(
//        pattern: &'_ [T],
//        index: usize,
//    ) -> Vec<T> {
//        pattern
//            .get(Self::front_context_range::<T>(index))
//            .unwrap_or(&[])
//            .to_vec()
//    }
//    fn next_child(
//        pattern: impl IntoPattern,
//        sub_index: usize,
//    ) -> Option<Child> {
//        Self::pattern_index_next(pattern.borrow(), sub_index)
//            .and_then(|i| pattern.borrow().get(i).map(ToChild::to_child))
//    }
//    fn compare_next_index_in_child_pattern(
//        child_pattern: impl IntoPattern,
//        context: impl IntoPattern,
//        sub_index: usize,
//    ) -> bool {
//        Self::pattern_head(context.borrow())
//            .and_then(|context_next| {
//                let context_next: Child = context_next.to_child();
//                Self::next_child(child_pattern, sub_index).map(|next| context_next == next)
//            })
//            .unwrap_or(false)
//    }
//}
