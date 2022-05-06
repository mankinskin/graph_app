use crate::{
    graph::*,
    vertex::*,
    search::*,
};
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
fn pattern_skip_offset(
    pattern: impl IntoPattern,
    mut offset: usize,
) -> Option<(usize, usize)> {
    pattern.into_iter()
        .enumerate()
        .find_map(|(i, c)|
            if c.width() > offset {
                Some((i, offset))
            } else {
                offset -= c.width();
                None
            }
        )
}
fn pattern_find_offset_end(
    pattern: impl IntoPattern,
    mut offset: usize,
) -> Option<(usize, usize)> {
    pattern.into_iter()
        .enumerate()
        .find_map(|(i, c)|
            match c.width().cmp(&offset) {
                Ordering::Greater => Some((i, offset)),
                Ordering::Equal => Some((i, 0)),
                Ordering::Less => {
                    offset -= c.width();
                    None
                }
            }
        )
}
pub trait MatchDirection : Clone {
    type Opposite: MatchDirection;
    /// get the parent where vertex is at the relevant position
    fn get_match_parent_to(
        graph: &Hypergraph<impl Tokenize>,
        vertex: &VertexData,
        sup: impl Indexed,
    ) -> Result<PatternIndex, NoMatch>;
    fn skip_equal_indices<'a, I: Indexed, J: Indexed>(
        a: impl DoubleEndedIterator<Item = &'a I>,
        b: impl DoubleEndedIterator<Item = &'a J>,
    ) -> Option<(TokenPosition, EitherOrBoth<&'a I, &'a J>)>;
    /// get remaining pattern in matching direction including index
    fn split_end<T: AsChild + Clone>(
        pattern: &'_ [T],
        index: PatternId,
    ) -> Vec<T>;
    /// get remaining pattern in matching direction excluding index
    fn front_context<T: AsChild + Clone>(
        pattern: &'_ [T],
        index: PatternId,
    ) -> Vec<T>;
    /// get remaining pattern agains matching direction excluding index
    fn back_context<T: AsChild + Clone>(
        pattern: &'_ [T],
        index: PatternId,
    ) -> Vec<T>;
    fn pattern_tail<T: AsChild>(pattern: &'_ [T]) -> &'_ [T];
    fn pattern_head<T: AsChild>(pattern: &'_ [T]) -> Option<&T>;
    fn head_index<T: AsChild>(_pattern: &'_ [T]) -> usize;
    fn last_index<T: AsChild>(pattern: &'_ [T]) -> usize {
        Self::Opposite::head_index(pattern)
    }
    fn normalize_index<T: AsChild>(
        pattern: &'_ [T],
        index: usize,
    ) -> usize;
    fn merge_remainder_with_context<
        A: IntoPattern,
        B: IntoPattern,
    >(
        rem: A,
        context: B,
    ) -> Pattern;
    fn index_next(
        index: usize,
    ) -> Option<usize>;
    fn index_prev(
        index: usize,
    ) -> Option<usize>;
    fn tail_index<T: AsChild>(pattern:  &'_ [T], tail: &'_ [T]) -> usize;
    /// filter pattern indices of parent relation by child patterns and matching direction
    fn filter_parent_pattern_indices(
        parent: &Parent,
        child_patterns: &HashMap<PatternId, Pattern>,
    ) -> HashSet<PatternIndex>;

    fn pattern_offset_context_split(
        pattern: impl IntoPattern,
        offset: usize,
    ) -> Option<(usize, usize)>;

    fn pattern_offset_inner_split(
        pattern: impl IntoPattern,
        offset: usize,
    ) -> Option<(usize, usize)>;

    fn split_head_tail<T: AsChild + Clone>(pattern: &'_ [T]) -> Option<(T, &'_ [T])> {
        Self::pattern_head(pattern).map(|head| (head.clone(), Self::pattern_tail(pattern)))
    }
    fn split_end_normalized<T: AsChild + Clone>(
        pattern: &'_ [T],
        index: PatternId,
    ) -> Vec<T> {
        Self::split_end(pattern, Self::normalize_index(pattern, index))
    }
    fn front_context_normalized<T: AsChild + Clone>(
        pattern: &'_ [T],
        index: PatternId,
    ) -> Vec<T> {
        Self::front_context(pattern, Self::normalize_index(pattern, index))
    }
    fn back_context_normalized<T: AsChild + Clone>(
        pattern: &'_ [T],
        index: PatternId,
    ) -> Vec<T> {
        Self::back_context(pattern, Self::normalize_index(pattern, index))
    }
    fn pattern_index_next(
        pattern: impl IntoPattern,
        index: usize,
    ) -> Option<usize> {
        Self::index_next(index).and_then(|i|
            (i < pattern.borrow().len()).then(|| i)
        )
    }
    fn pattern_index_prev(
        pattern: impl IntoPattern,
        index: usize,
    ) -> Option<usize> {
        Self::index_prev(index).and_then(|i|
            (i < pattern.borrow().len()).then(|| i)
        )
    }
    //fn to_found_range(
    //    p: Option<Pattern>,
    //    context: Pattern,
    //) -> FoundRange;
    //fn found_from_start(fr: &FoundRange) -> bool;
    //fn found_till_end(fr: &FoundRange) -> bool;
    //fn get_remainder(found_range: FoundRange) -> Option<Pattern>;
    fn directed_pattern_split<T: AsChild + Clone>(
        pattern: &'_ [T],
        index: usize,
    ) -> (Vec<T>, Vec<T>) {
        (
            Self::back_context(pattern, index),
            Self::split_end_normalized(pattern, index),
        )
    }
    fn next_child(
        pattern: impl IntoPattern,
        sub_index: usize,
    ) -> Option<Child> {
        Self::pattern_index_next(pattern.borrow(), sub_index).and_then(|i| {
            pattern.borrow().get(i).map(AsChild::as_child)
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
    ) -> Result<PatternIndex, NoMatch> {
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
    fn pattern_offset_context_split(
        pattern: impl IntoPattern,
        offset: usize,
    ) -> Option<(usize, usize)> {
        pattern_skip_offset(pattern, offset)
    }

    fn pattern_offset_inner_split(
        pattern: impl IntoPattern,
        offset: usize,
    ) -> Option<(usize, usize)> {
        pattern_find_offset_end(pattern, offset)
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
    fn tail_index<T: AsChild>(pattern:  &'_ [T], tail: &'_ [T]) -> usize {
        pattern.len() - tail.len() - 1
    }
    fn index_next(
        index: usize,
    ) -> Option<usize> {
        index.checked_add(1)
    }
    fn index_prev(
        index: usize,
    ) -> Option<usize> {
        index.checked_sub(1)
    }
    fn normalize_index<T: AsChild>(
        _pattern: &'_ [T],
        index: usize,
    ) -> usize {
        index
    }
    fn merge_remainder_with_context<
        A: IntoPattern,
        B: IntoPattern,
    >(
        rem: A,
        context: B,
    ) -> Pattern {
        [rem.borrow(), context.borrow()].concat()
    }
    fn filter_parent_pattern_indices(
        parent: &Parent,
        _patterns: &HashMap<PatternId, Pattern>,
    ) -> HashSet<PatternIndex> {
        parent
            .filter_pattern_indices_at_prefix()
            .cloned()
            .collect()
    }
}

impl MatchDirection for Left {
    type Opposite = Left;
    fn get_match_parent_to(
        graph: &Hypergraph<impl Tokenize>,
        vertex: &VertexData,
        sup: impl Indexed,
    ) -> Result<PatternIndex, NoMatch> {
        let sup = graph.expect_vertex_data(sup);
        vertex.get_parent_at_postfix_of(sup)
    }
    fn skip_equal_indices<'a, I: Indexed, J: Indexed>(
        a: impl DoubleEndedIterator<Item = &'a I>,
        b: impl DoubleEndedIterator<Item = &'a J>,
    ) -> Option<(TokenPosition, EitherOrBoth<&'a I, &'a J>)> {
        to_matching_iterator(a.rev(), b.rev()).next()
    }
    fn pattern_offset_context_split(
        pattern: impl IntoPattern,
        offset: usize,
    ) -> Option<(usize, usize)> {
        pattern_find_offset_end(pattern, offset)
    }

    fn pattern_offset_inner_split(
        pattern: impl IntoPattern,
        offset: usize,
    ) -> Option<(usize, usize)> {
        pattern_skip_offset(pattern, offset)
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
    fn tail_index<T: AsChild>(_pattern:  &'_ [T], tail: &'_ [T]) -> usize {
        tail.len()
    }
    fn index_next(
        index: usize
    ) -> Option<usize> {
        index.checked_sub(1)
    }
    fn index_prev(
        index: usize,
    ) -> Option<usize> {
        index.checked_add(1)
    }
    fn normalize_index<T: AsChild>(
        pattern: &'_ [T],
        index: usize,
    ) -> usize {
        pattern.len() - index - 1
    }
    fn merge_remainder_with_context<
        A: IntoPattern,
        B: IntoPattern,
    >(
        rem: A,
        context: B,
    ) -> Pattern {
        [context.borrow(), rem.borrow()].concat()
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
    //fn to_found_range(
    //    p: Option<Pattern>,
    //    context: Pattern,
    //) -> FoundRange {
    //    match (context.is_empty(), p) {
    //        (false, Some(rem)) => FoundRange::Infix(rem, context),
    //        (true, Some(rem)) => FoundRange::Postfix(rem),
    //        (false, None) => FoundRange::Prefix(context),
    //        (true, None) => FoundRange::Complete,
    //    }
    //}
    //fn get_remainder(found_range: FoundRange) -> Option<Pattern> {
    //    match found_range {
    //        FoundRange::Postfix(rem) => Some(rem),
    //        _ => None,
    //    }
    //}
    //fn found_from_start(fr: &FoundRange) -> bool {
    //    matches!(fr, FoundRange::Postfix(_) | FoundRange::Complete)
    //}
    //fn found_till_end(fr: &FoundRange) -> bool {
    //    matches!(fr, FoundRange::Prefix(_) | FoundRange::Complete)
    //}
}
