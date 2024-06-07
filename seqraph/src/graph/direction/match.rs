use std::{
    fmt::Debug,
    ops::{
        Range,
        RangeFrom,
    },
};

use itertools::{
    EitherOrBoth,
    Itertools,
};

use crate::{
    direction::{
        Left,
        Right,
    },
    graph::GraphKind,
    search::NoMatch,
    vertex::{
        child::Child,
        indexed::ToChild,
        parent::PatternIndex,
        pattern::{
            pattern_range::PatternRangeIndex,
            IntoPattern,
            Pattern,
        },
        PatternId,
        TokenPosition,
    },
    HashMap,
    HashSet,
};

fn to_matching_iterator<
    'a,
    I: crate::vertex::indexed::Indexed + 'a,
    J: crate::vertex::indexed::Indexed + 'a,
>(
    a: impl Iterator<Item = &'a I>,
    b: impl Iterator<Item = &'a J>,
) -> impl Iterator<Item = (usize, EitherOrBoth<&'a I, &'a J>)> {
    a.zip_longest(b)
        .enumerate()
        .skip_while(|(_, eob)| match eob {
            EitherOrBoth::Both(a, b) => a.vertex_index() == b.vertex_index(),
            _ => false,
        })
}

pub trait MatchDirection: Clone + Debug + Send + Sync + 'static + Unpin {
    type Opposite: MatchDirection;
    type PostfixRange<T>: PatternRangeIndex<T>;
    /// get the parent where vertex is at the relevant position
    fn get_match_parent_to<G: GraphKind>(
        graph: &crate::graph::Hypergraph<G>,
        vertex: &crate::vertex::VertexData<G>,
        sup: impl crate::vertex::indexed::Indexed,
    ) -> Result<PatternIndex, NoMatch>;
    fn skip_equal_indices<
        'a,
        I: crate::vertex::indexed::Indexed,
        J: crate::vertex::indexed::Indexed,
    >(
        a: impl DoubleEndedIterator<Item = &'a I>,
        b: impl DoubleEndedIterator<Item = &'a J>,
    ) -> Option<(TokenPosition, EitherOrBoth<&'a I, &'a J>)>;
    /// get remaining pattern in matching direction including index
    fn pattern_tail<T: crate::vertex::indexed::AsChild>(pattern: &'_ [T]) -> &'_ [T];
    fn pattern_head<T: crate::vertex::indexed::AsChild>(pattern: &'_ [T]) -> Option<&'_ T>;
    fn head_index(pattern: impl IntoPattern) -> usize;
    fn last_index(pattern: impl IntoPattern) -> usize {
        Self::Opposite::head_index(pattern)
    }
    fn merge_remainder_with_context<A: IntoPattern, B: IntoPattern>(
        rem: A,
        context: B,
    ) -> Pattern;
    fn index_next(index: usize) -> Option<usize>;
    fn index_prev(index: usize) -> Option<usize>;
    fn tail_index<T: crate::vertex::indexed::AsChild>(
        pattern: &'_ [T],
        tail: &'_ [T],
    ) -> usize;
    /// filter pattern indices of parent relation by child patterns and matching direction
    fn filter_parent_pattern_indices(
        parent: &crate::vertex::parent::Parent,
        child_patterns: &HashMap<PatternId, Pattern>,
    ) -> HashSet<PatternIndex>;
    fn split_head_tail<T: crate::vertex::indexed::AsChild + Clone>(
        pattern: &'_ [T]
    ) -> Option<(T, &'_ [T])> {
        Self::pattern_head(pattern).map(|head| (head.clone(), Self::pattern_tail(pattern)))
    }
    fn front_context_range<T>(index: PatternId) -> Self::PostfixRange<T>;
    /// get remaining pattern in matching direction excluding index
    fn front_context<T: crate::vertex::indexed::AsChild + Clone>(
        pattern: &'_ [T],
        index: PatternId,
    ) -> Vec<T> {
        pattern
            .get(Self::front_context_range::<T>(index))
            .unwrap_or(&[])
            .to_vec()
    }
    //fn front_context_normalized<T: AsChild + Clone>(
    //    pattern: &'_ [T],
    //    index: PatternId,
    //) -> Vec<T> {
    //    Self::front_context(pattern, Self::normalize_index(pattern, index))
    //}
    //fn back_context_normalized<T: AsChild + Clone>(
    //    pattern: &'_ [T],
    //    index: PatternId,
    //) -> Vec<T> {
    //    Self::back_context(pattern, Self::normalize_index(pattern, index))
    //}
    fn pattern_index_next(
        pattern: impl IntoPattern,
        index: usize,
    ) -> Option<usize> {
        Self::index_next(index).and_then(|i| (i < pattern.borrow().len()).then(|| i))
    }
    fn pattern_index_prev(
        pattern: impl IntoPattern,
        index: usize,
    ) -> Option<usize> {
        Self::index_prev(index).and_then(|i| (i < pattern.borrow().len()).then(|| i))
    }
    fn next_child(
        pattern: impl IntoPattern,
        sub_index: usize,
    ) -> Option<Child> {
        Self::pattern_index_next(pattern.borrow(), sub_index).and_then(|i| {
            pattern
                .borrow()
                .get(i)
                .map(crate::vertex::indexed::AsChild::as_child)
        })
    }
    fn compare_next_index_in_child_pattern(
        child_pattern: impl IntoPattern,
        context: impl IntoPattern,
        sub_index: usize,
    ) -> bool {
        Self::pattern_head(context.borrow())
            .and_then(|context_next| {
                let context_next: Child = context_next.to_child();
                Self::next_child(child_pattern, sub_index).map(|next| context_next == next)
            })
            .unwrap_or(false)
    }
}

impl MatchDirection for Right {
    type Opposite = Left;
    type PostfixRange<T> = RangeFrom<PatternId>;
    fn get_match_parent_to<G: GraphKind>(
        _graph: &crate::graph::Hypergraph<G>,
        vertex: &crate::vertex::VertexData<G>,
        sup: impl crate::vertex::indexed::Indexed,
    ) -> Result<PatternIndex, NoMatch> {
        vertex.get_parent_at_prefix_of(sup)
    }
    fn skip_equal_indices<
        'a,
        I: crate::vertex::indexed::Indexed,
        J: crate::vertex::indexed::Indexed,
    >(
        a: impl DoubleEndedIterator<Item = &'a I>,
        b: impl DoubleEndedIterator<Item = &'a J>,
    ) -> Option<(TokenPosition, EitherOrBoth<&'a I, &'a J>)> {
        to_matching_iterator(a, b).next()
    }

    fn front_context_range<T>(index: PatternId) -> Self::PostfixRange<T> {
        (index + 1)..
    }
    fn pattern_tail<T: crate::vertex::indexed::AsChild>(pattern: &'_ [T]) -> &'_ [T] {
        pattern.get(1..).unwrap_or(&[])
    }
    fn pattern_head<T: crate::vertex::indexed::AsChild>(pattern: &'_ [T]) -> Option<&'_ T> {
        pattern.first()
    }
    fn head_index(_pattern: impl IntoPattern) -> usize {
        0
    }
    fn tail_index<T: crate::vertex::indexed::AsChild>(
        pattern: &'_ [T],
        tail: &'_ [T],
    ) -> usize {
        pattern.len() - tail.len() - 1
    }
    fn index_next(index: usize) -> Option<usize> {
        index.checked_add(1)
    }
    fn index_prev(index: usize) -> Option<usize> {
        index.checked_sub(1)
    }
    fn merge_remainder_with_context<A: IntoPattern, B: IntoPattern>(
        rem: A,
        context: B,
    ) -> Pattern {
        [rem.borrow(), context.borrow()].concat()
    }
    fn filter_parent_pattern_indices(
        parent: &crate::vertex::parent::Parent,
        _patterns: &HashMap<PatternId, Pattern>,
    ) -> HashSet<PatternIndex> {
        parent.filter_pattern_indices_at_prefix().cloned().collect()
    }
}

impl MatchDirection for Left {
    type Opposite = Left;
    type PostfixRange<T> = Range<PatternId>;
    fn get_match_parent_to<G: GraphKind>(
        graph: &crate::graph::Hypergraph<G>,
        vertex: &crate::vertex::VertexData<G>,
        sup: impl crate::vertex::indexed::Indexed,
    ) -> Result<PatternIndex, NoMatch> {
        let sup = graph.expect_vertex_data(sup);
        vertex.get_parent_at_postfix_of(sup)
    }
    fn skip_equal_indices<
        'a,
        I: crate::vertex::indexed::Indexed,
        J: crate::vertex::indexed::Indexed,
    >(
        a: impl DoubleEndedIterator<Item = &'a I>,
        b: impl DoubleEndedIterator<Item = &'a J>,
    ) -> Option<(TokenPosition, EitherOrBoth<&'a I, &'a J>)> {
        to_matching_iterator(a.rev(), b.rev()).next()
    }
    fn front_context_range<T>(index: PatternId) -> Self::PostfixRange<T> {
        0..index
    }
    fn pattern_tail<T: crate::vertex::indexed::AsChild>(pattern: &'_ [T]) -> &'_ [T] {
        pattern.split_last().map(|(_last, pre)| pre).unwrap_or(&[])
    }
    fn pattern_head<T: crate::vertex::indexed::AsChild>(pattern: &'_ [T]) -> Option<&'_ T> {
        pattern.last()
    }
    fn head_index(pattern: impl IntoPattern) -> usize {
        pattern.borrow().len() - 1
    }
    fn tail_index<T: crate::vertex::indexed::AsChild>(
        _pattern: &'_ [T],
        tail: &'_ [T],
    ) -> usize {
        tail.len()
    }
    fn index_next(index: usize) -> Option<usize> {
        index.checked_sub(1)
    }
    fn index_prev(index: usize) -> Option<usize> {
        index.checked_add(1)
    }
    fn merge_remainder_with_context<A: IntoPattern, B: IntoPattern>(
        rem: A,
        context: B,
    ) -> Pattern {
        [context.borrow(), rem.borrow()].concat()
    }
    fn filter_parent_pattern_indices(
        parent: &crate::vertex::parent::Parent,
        child_patterns: &HashMap<PatternId, Pattern>,
    ) -> HashSet<PatternIndex> {
        parent
            .filter_pattern_indices_at_end_in_patterns(child_patterns)
            .cloned()
            .collect()
    }
}
