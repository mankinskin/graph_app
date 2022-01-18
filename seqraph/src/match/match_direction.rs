use crate::{r#match::*};
use itertools::{
    EitherOrBoth,
    Itertools,
};
use std::collections::{
    HashMap,
    HashSet,
};

fn to_matching_iterator<'a, I: Indexed + 'a, J: Indexed + 'a>(
    a: impl Iterator<Item = &'a I>,
    b: impl Iterator<Item = &'a J>,
) -> impl Iterator<Item = (usize, EitherOrBoth<&'a I, &'a J>)> {
    a.zip_longest(b)
        .enumerate()
        .skip_while(|(_, eob)| match eob {
            EitherOrBoth::Both(a, b) => a.index() == b.index(),
            _ => false,
        })
}
pub trait MatchDirection {
    type Opposite: MatchDirection;
    /// get the parent where vertex is at the relevant position
    fn get_match_parent_to(
        graph: &Hypergraph<impl Tokenize>,
        vertex: &VertexData,
        sup: impl Indexed,
    ) -> Result<(PatternId, usize), NoMatch>;
    fn skip_equal_indices<'a, I: Indexed, J: Indexed>(
        a: impl DoubleEndedIterator<Item = &'a I>,
        b: impl DoubleEndedIterator<Item = &'a J>,
    ) -> Option<(TokenPosition, EitherOrBoth<&'a I, &'a J>)>;
    fn split_head_tail<T: AsChild + Clone>(pattern: &'_ [T]) -> Option<(T, &'_ [T])> {
        Self::pattern_head(pattern).map(|head| (head.clone(), Self::pattern_tail(pattern)))
    }
    /// get remaining pattern in matching direction including index
    fn split_end<T: AsChild + Clone>(
        pattern: &'_ [T],
        index: PatternId,
    ) -> Vec<T>;
    fn split_end_normalized<T: AsChild + Clone>(
        pattern: &'_ [T],
        index: PatternId,
    ) -> Vec<T> {
        Self::split_end(pattern, Self::normalize_index(pattern, index))
    }
    /// get remaining pattern in matching direction excluding index
    fn front_context<T: AsChild + Clone>(
        pattern: &'_ [T],
        index: PatternId,
    ) -> Vec<T>;
    fn front_context_normalized<T: AsChild + Clone>(
        pattern: &'_ [T],
        index: PatternId,
    ) -> Vec<T> {
        Self::front_context(pattern, Self::normalize_index(pattern, index))
    }
    /// get remaining pattern agains matching direction excluding index
    fn back_context<T: AsChild + Clone>(
        pattern: &'_ [T],
        index: PatternId,
    ) -> Vec<T>;
    fn back_context_normalized<T: AsChild + Clone>(
        pattern: &'_ [T],
        index: PatternId,
    ) -> Vec<T> {
        Self::back_context(pattern, Self::normalize_index(pattern, index))
    }
    fn pattern_tail<T: AsChild>(pattern: &'_ [T]) -> &'_ [T];
    fn pattern_head<T: AsChild>(pattern: &'_ [T]) -> Option<&T>;
    fn head_index<T: AsChild>(_pattern: &'_ [T]) -> usize;
    fn normalize_index<T: AsChild>(
        pattern: &'_ [T],
        index: usize,
    ) -> usize;
    fn merge_remainder_with_context<
        T: AsChild + Clone,
        A: IntoPattern<Item = impl AsChild, Token = T>,
        B: IntoPattern<Item = impl AsChild, Token = T>,
    >(
        rem: A,
        context: B,
    ) -> Vec<T>;
    fn index_next(index: usize) -> Option<usize>;
    /// filter pattern indices of parent relation by child patterns and matching direction
    fn filter_parent_pattern_indices(
        parent: &Parent,
        child_patterns: &HashMap<PatternId, Pattern>,
    ) -> HashSet<(PatternId, usize)>;
    fn to_found_range(
        p: Option<Pattern>,
        context: Pattern,
    ) -> FoundRange;
    fn found_from_start(fr: &FoundRange) -> bool;
    fn found_till_end(fr: &FoundRange) -> bool;
    fn get_remainder(found_range: FoundRange) -> Option<Pattern>;
    fn directed_pattern_split<T: AsChild + Clone>(
        pattern: &'_ [T],
        index: usize,
    ) -> (Vec<T>, Vec<T>) {
        (
            Self::back_context(pattern, index),
            Self::split_end(pattern, index),
        )
    }
    fn next_child(
        pattern: impl IntoPattern<Item = impl AsChild>,
        sub_index: usize,
    ) -> Option<Child> {
        Self::index_next(sub_index).and_then(|i| {
            pattern.as_pattern_view().get(i).map(AsChild::as_child)
        })
    }
    fn compare_next_index_in_child_pattern(
        child_pattern: impl IntoPattern<Item = impl AsChild>,
        context: impl IntoPattern<Item = impl AsChild>,
        sub_index: usize,
    ) -> bool {
        Self::pattern_head(context.as_pattern_view())
            .and_then(|context_next| {
                let context_next: Child = context_next.to_child();
                Self::next_child(child_pattern, sub_index)
                    .map(|next| context_next == next)
            })
            .unwrap_or(false)
    }
}
impl MatchDirection for Right {
    type Opposite = Left;
    fn get_match_parent_to(
        _graph: &Hypergraph<impl Tokenize>,
        vertex: &VertexData,
        sup: impl Indexed,
    ) -> Result<(PatternId, usize), NoMatch> {
        vertex.get_parent_at_prefix_of(sup)
    }
    fn skip_equal_indices<'a, I: Indexed, J: Indexed>(
        a: impl DoubleEndedIterator<Item = &'a I>,
        b: impl DoubleEndedIterator<Item = &'a J>,
    ) -> Option<(TokenPosition, EitherOrBoth<&'a I, &'a J>)> {
        to_matching_iterator(a, b).next()
    }
    fn split_end<T: AsChild + Clone>(
        pattern: &'_ [T],
        index: PatternId,
    ) -> Vec<T> {
        postfix(pattern, index)
    }
    fn front_context<T: AsChild + Clone>(
        pattern: &'_ [T],
        index: PatternId,
    ) -> Vec<T> {
        postfix(pattern, index + 1)
    }
    fn back_context<T: AsChild + Clone>(
        pattern: &'_ [T],
        index: PatternId,
    ) -> Vec<T> {
        prefix(pattern, index)
    }
    fn pattern_tail<T: AsChild>(pattern: &'_ [T]) -> &'_ [T] {
        pattern.get(1..).unwrap_or(&[])
    }
    fn pattern_head<T: AsChild>(pattern: &'_ [T]) -> Option<&T> {
        pattern.first()
    }
    fn head_index<T: AsChild>(_pattern: &'_ [T]) -> usize {
        0
    }
    fn index_next(index: usize) -> Option<usize> {
        index.checked_add(1)
    }
    fn normalize_index<T: AsChild>(
        _pattern: &'_ [T],
        index: usize,
    ) -> usize {
        index
    }
    fn merge_remainder_with_context<
        T: AsChild + Clone,
        A: IntoPattern<Item = impl AsChild, Token = T>,
        B: IntoPattern<Item = impl AsChild, Token = T>,
    >(
        rem: A,
        context: B,
    ) -> Vec<T> {
        [rem.as_pattern_view(), context.as_pattern_view()].concat()
    }
    fn filter_parent_pattern_indices(
        parent: &Parent,
        _patterns: &HashMap<PatternId, Pattern>,
    ) -> HashSet<(PatternId, usize)> {
        parent
            .filter_pattern_indices_at_prefix()
            .cloned()
            .collect()
    }
    fn to_found_range(
        p: Option<Pattern>,
        context: Pattern,
    ) -> FoundRange {
        match (context.is_empty(), p) {
            (false, Some(rem)) => FoundRange::Infix(context, rem),
            (true, Some(rem)) => FoundRange::Prefix(rem),
            (false, None) => FoundRange::Postfix(context),
            (true, None) => FoundRange::Complete,
        }
    }
    fn get_remainder(found_range: FoundRange) -> Option<Pattern> {
        match found_range {
            FoundRange::Prefix(rem) => Some(rem),
            _ => None,
        }
    }
    fn found_from_start(fr: &FoundRange) -> bool {
        matches!(fr, FoundRange::Prefix(_) | FoundRange::Complete)
    }
    fn found_till_end(fr: &FoundRange) -> bool {
        matches!(fr, FoundRange::Postfix(_) | FoundRange::Complete)
    }
}

impl MatchDirection for Left {
    type Opposite = Left;
    fn get_match_parent_to(
        graph: &Hypergraph<impl Tokenize>,
        vertex: &VertexData,
        sup: impl Indexed,
    ) -> Result<(PatternId, usize), NoMatch> {
        let sup = graph.expect_vertex_data(sup);
        vertex.get_parent_at_postfix_of(sup)
    }
    fn skip_equal_indices<'a, I: Indexed, J: Indexed>(
        a: impl DoubleEndedIterator<Item = &'a I>,
        b: impl DoubleEndedIterator<Item = &'a J>,
    ) -> Option<(TokenPosition, EitherOrBoth<&'a I, &'a J>)> {
        to_matching_iterator(a.rev(), b.rev()).next()
    }
    fn split_end<T: AsChild + Clone>(
        pattern: &'_ [T],
        index: PatternId,
    ) -> Vec<T> {
        prefix(pattern, index + 1)
    }
    fn front_context<T: AsChild + Clone>(
        pattern: &'_ [T],
        index: PatternId,
    ) -> Vec<T> {
        prefix(pattern, index)
    }
    fn back_context<T: AsChild + Clone>(
        pattern: &'_ [T],
        index: PatternId,
    ) -> Vec<T> {
        postfix(pattern, index + 1)
    }
    fn pattern_tail<T: AsChild>(pattern: &'_ [T]) -> &'_ [T] {
        pattern.split_last().map(|(_last, pre)| pre).unwrap_or(&[])
    }
    fn pattern_head<T: AsChild>(pattern: &'_ [T]) -> Option<&T> {
        pattern.last()
    }
    fn head_index<T: AsChild>(pattern: &'_ [T]) -> usize {
        pattern.len() - 1
    }
    fn index_next(index: usize) -> Option<usize> {
        index.checked_sub(1)
    }
    fn normalize_index<T: AsChild>(
        pattern: &'_ [T],
        index: usize,
    ) -> usize {
        pattern.len() - index - 1
    }
    fn merge_remainder_with_context<
        T: AsChild + Clone,
        A: IntoPattern<Item = impl AsChild, Token = T>,
        B: IntoPattern<Item = impl AsChild, Token = T>,
    >(
        rem: A,
        context: B,
    ) -> Vec<T> {
        [context.as_pattern_view(), rem.as_pattern_view()].concat()
    }
    fn filter_parent_pattern_indices(
        parent: &Parent,
        child_patterns: &HashMap<PatternId, Pattern>,
    ) -> HashSet<(PatternId, usize)> {
        parent
            .filter_pattern_indices_at_end_in_patterns(child_patterns)
            .cloned()
            .collect()
    }
    fn to_found_range(
        p: Option<Pattern>,
        context: Pattern,
    ) -> FoundRange {
        match (context.is_empty(), p) {
            (false, Some(rem)) => FoundRange::Infix(rem, context),
            (true, Some(rem)) => FoundRange::Postfix(rem),
            (false, None) => FoundRange::Prefix(context),
            (true, None) => FoundRange::Complete,
        }
    }
    fn get_remainder(found_range: FoundRange) -> Option<Pattern> {
        match found_range {
            FoundRange::Postfix(rem) => Some(rem),
            _ => None,
        }
    }
    fn found_from_start(fr: &FoundRange) -> bool {
        matches!(fr, FoundRange::Postfix(_) | FoundRange::Complete)
    }
    fn found_till_end(fr: &FoundRange) -> bool {
        matches!(fr, FoundRange::Prefix(_) | FoundRange::Complete)
    }
}
