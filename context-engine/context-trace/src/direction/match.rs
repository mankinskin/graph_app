use std::ops::{
    Range,
    RangeFrom,
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
    graph::{
        getters::{
            vertex::VertexSet,
            ErrorReason,
        },
        kind::GraphKind,
        vertex::{
            child::Child,
            data::VertexData,
            has_vertex_index::{
                HasVertexIndex,
                ToChild,
            },
            parent::{
                Parent,
                PatternIndex,
            },
            pattern::{
                id::PatternId,
                pattern_range::PatternRangeIndex,
                Pattern,
            },
        },
        Hypergraph,
    },
    HashMap,
    HashSet,
};

use super::Direction;

fn to_matching_iterator<'a, I: HasVertexIndex + 'a, J: HasVertexIndex + 'a>(
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

pub trait MatchDirection: Direction {
    type PostfixRange<T>: PatternRangeIndex<T>;
    /// get the parent where vertex is at the relevant position
    fn get_match_parent_to<G: GraphKind>(
        graph: &Hypergraph<G>,
        vertex: &VertexData,
        sup: impl HasVertexIndex,
    ) -> Result<PatternIndex, ErrorReason>;
    fn skip_equal_indices<'a, I: HasVertexIndex, J: HasVertexIndex>(
        a: impl DoubleEndedIterator<Item = &'a I>,
        b: impl DoubleEndedIterator<Item = &'a J>,
    ) -> Option<(usize, EitherOrBoth<&'a I, &'a J>)>;
    /// get remaining pattern in matching direction including index
    fn pattern_tail<T: ToChild>(pattern: &'_ [T]) -> &'_ [T];
    fn pattern_head<T: ToChild>(pattern: &'_ [T]) -> Option<&'_ T>;
    fn head_index(pattern: &Pattern) -> usize;
    fn last_index(pattern: &Pattern) -> usize
    where
        <Self as Direction>::Opposite: MatchDirection,
    {
        Self::Opposite::head_index(pattern)
    }
    fn merge_remainder_with_context(
        rem: &Pattern,
        context: &Pattern,
    ) -> Pattern;
    fn index_next(index: usize) -> Option<usize>;
    fn index_prev(index: usize) -> Option<usize>;
    fn tail_index<T: ToChild>(
        pattern: &'_ [T],
        tail: &'_ [T],
    ) -> usize;
    /// filter pattern indices of parent relation by child patterns and matching direction
    fn filter_parent_pattern_indices(
        parent: &Parent,
        child_patterns: &HashMap<PatternId, Pattern>,
    ) -> HashSet<PatternIndex>;
    fn split_head_tail<T: ToChild + Clone>(pattern: &'_ [T]) -> Option<(T, &'_ [T])> {
        Self::pattern_head(pattern).map(|head| (head.clone(), Self::pattern_tail(pattern)))
    }
    fn front_context_range<T>(index: usize) -> Self::PostfixRange<T>;
    /// get remaining pattern in matching direction excluding index
    fn front_context<T: ToChild + Clone>(
        pattern: &'_ [T],
        index: usize,
    ) -> Vec<T> {
        pattern
            .get(Self::front_context_range::<T>(index))
            .unwrap_or(&[])
            .to_vec()
    }
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
    fn next_child(
        pattern: &Pattern,
        sub_index: usize,
    ) -> Option<Child> {
        Self::pattern_index_next(pattern, sub_index)
            .and_then(|i| pattern.get(i).map(ToChild::to_child))
    }
    fn compare_next_index_in_child_pattern(
        child_pattern: &Pattern,
        context: &Pattern,
        sub_index: usize,
    ) -> bool {
        Self::pattern_head(context)
            .and_then(|context_next| {
                let context_next: Child = context_next.to_child();
                Self::next_child(child_pattern, sub_index).map(|next| context_next == next)
            })
            .unwrap_or(false)
    }
}

impl MatchDirection for Right {
    type PostfixRange<T> = RangeFrom<usize>;
    fn get_match_parent_to<G: GraphKind>(
        _graph: &Hypergraph<G>,
        vertex: &VertexData,
        sup: impl HasVertexIndex,
    ) -> Result<PatternIndex, ErrorReason> {
        vertex.get_parent_at_prefix_of(sup)
    }
    fn skip_equal_indices<'a, I: HasVertexIndex, J: HasVertexIndex>(
        a: impl DoubleEndedIterator<Item = &'a I>,
        b: impl DoubleEndedIterator<Item = &'a J>,
    ) -> Option<(usize, EitherOrBoth<&'a I, &'a J>)> {
        to_matching_iterator(a, b).next()
    }

    fn front_context_range<T>(index: usize) -> Self::PostfixRange<T> {
        (index + 1)..
    }
    fn pattern_tail<T: ToChild>(pattern: &'_ [T]) -> &'_ [T] {
        pattern.get(1..).unwrap_or(&[])
    }
    fn pattern_head<T: ToChild>(pattern: &'_ [T]) -> Option<&'_ T> {
        pattern.first()
    }
    fn head_index(_pattern: &Pattern) -> usize {
        0
    }
    fn tail_index<T: ToChild>(
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
    fn merge_remainder_with_context(
        rem: &Pattern,
        context: &Pattern,
    ) -> Pattern {
        [rem.clone(), context.clone()].concat()
    }
    fn filter_parent_pattern_indices(
        parent: &Parent,
        _patterns: &HashMap<PatternId, Pattern>,
    ) -> HashSet<PatternIndex> {
        parent.filter_pattern_indices_at_prefix().cloned().collect()
    }
}

impl MatchDirection for Left {
    type PostfixRange<T> = Range<usize>;
    fn get_match_parent_to<G: GraphKind>(
        graph: &Hypergraph<G>,
        vertex: &VertexData,
        sup: impl HasVertexIndex,
    ) -> Result<PatternIndex, ErrorReason> {
        let sup = graph.expect_vertex(sup.vertex_index());
        vertex.get_parent_at_postfix_of(sup)
    }
    fn skip_equal_indices<'a, I: HasVertexIndex, J: HasVertexIndex>(
        a: impl DoubleEndedIterator<Item = &'a I>,
        b: impl DoubleEndedIterator<Item = &'a J>,
    ) -> Option<(usize, EitherOrBoth<&'a I, &'a J>)> {
        to_matching_iterator(a.rev(), b.rev()).next()
    }
    fn front_context_range<T>(index: usize) -> Self::PostfixRange<T> {
        0..index
    }
    fn pattern_tail<T: ToChild>(pattern: &'_ [T]) -> &'_ [T] {
        pattern.split_last().map(|(_last, pre)| pre).unwrap_or(&[])
    }
    fn pattern_head<T: ToChild>(pattern: &'_ [T]) -> Option<&'_ T> {
        pattern.last()
    }
    fn head_index(pattern: &Pattern) -> usize {
        pattern.len() - 1
    }
    fn tail_index<T: ToChild>(
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
    fn merge_remainder_with_context(
        rem: &Pattern,
        context: &Pattern,
    ) -> Pattern {
        [context.clone(), rem.clone()].concat()
    }
    fn filter_parent_pattern_indices(
        parent: &Parent,
        child_patterns: &HashMap<PatternId, Pattern>,
    ) -> HashSet<PatternIndex> {
        parent
            .filter_pattern_indices_at_end_in_patterns(child_patterns)
            .cloned()
            .collect()
    }
}
