use std::{
    borrow::{
        Borrow,
        BorrowMut,
    },
    fmt::Debug,
    iter::IntoIterator,
};

use crate::graph::vertex::{
    pattern::pattern_range::PatternRangeIndex,
    wide::Wide,
};

use super::{
    child::Child,
    has_vertex_index::ToChild,
};

pub mod id;
pub mod pattern_range;

pub type Pattern = Vec<Child>;
pub type PatternView<'a> = &'a [Child];
pub type Patterns = Vec<Pattern>;

/// trait for types which can be converted to a pattern with a known size
pub trait IntoPattern: Sized
//IntoIterator<Item = Self::Elem, IntoIter = Self::Iter> + Sized + Borrow<[Child]> + Debug
{
    //type Iter: ExactSizeIterator + DoubleEndedIterator<Item = Self::Elem>;
    //type Elem: ToChild;

    //fn into_pattern(self) -> Pattern {
    //    self.into_iter().map(|x| x.to_child()).collect()
    //}
    fn into_pattern(self) -> Pattern;
    fn is_empty(&self) -> bool;
}

impl<const N: usize> IntoPattern for [Child; N] {
    fn into_pattern(self) -> Pattern {
        self.into_iter().collect()
    }
    fn is_empty(&self) -> bool {
        N == 0
    }
}
impl IntoPattern for Child {
    fn into_pattern(self) -> Pattern {
        Some(self).into_iter().collect()
    }
    fn is_empty(&self) -> bool {
        false
    }
}
//impl<It: IntoIterator<Item = Child> + Borrow<[Child]>> IntoPattern for It {
//    fn into_pattern(self) -> Pattern {
//        self.into_iter().collect()
//    }
//    fn is_empty(&self) -> bool {
//        (*self).borrow().is_empty()
//    }
//}
impl IntoPattern for &'_ [Child] {
    fn into_pattern(self) -> Pattern {
        self.iter().map(Clone::clone).collect()
    }
    fn is_empty(&self) -> bool {
        (*self).is_empty()
    }
}
impl IntoPattern for Pattern {
    fn into_pattern(self) -> Pattern {
        self
    }
    fn is_empty(&self) -> bool {
        (*self).is_empty()
    }
}
impl<T: IntoPattern + Clone> IntoPattern for &'_ T {
    fn into_pattern(self) -> Pattern {
        self.clone().into_pattern()
    }
    fn is_empty(&self) -> bool {
        (*self).is_empty()
    }
}

//impl<C, It, T> IntoPattern for T
//where
//    C: ToChild,
//    It: DoubleEndedIterator<Item = C> + ExactSizeIterator,
//    T: IntoIterator<Item = C, IntoIter = It> + Borrow<[Child]> + Debug,
//{
//    type Iter = It;
//    type Elem = C;
//}

/// trait for types which can be converted to a pattern with a known size
pub trait AsPatternMut: BorrowMut<Vec<Child>> + Debug {}

impl<T> AsPatternMut for T where T: BorrowMut<Vec<Child>> + Debug {}

pub fn pattern_width<T: Borrow<Child>>(
    pat: impl IntoIterator<Item = T>
) -> usize {
    pat.into_iter().map(|c| c.borrow().width()).sum()
}

pub fn pattern_pre_ctx<T: Borrow<Child>>(
    pat: impl IntoIterator<Item = T>,
    sub_index: usize,
) -> impl IntoIterator<Item = T> {
    pat.into_iter().take(sub_index)
}

pub fn pattern_post_ctx<T: Borrow<Child>>(
    pat: impl IntoIterator<Item = T>,
    sub_index: usize,
) -> impl IntoIterator<Item = T> {
    pat.into_iter().skip(sub_index + 1)
}

pub fn prefix<T: ToChild + Clone>(
    pattern: &'_ [T],
    index: usize,
) -> Vec<T> {
    pattern.get(0..index).unwrap_or(pattern).to_vec()
}

pub fn infix<T: ToChild + Clone>(
    pattern: &'_ [T],
    start: usize,
    end: usize,
) -> Vec<T> {
    pattern.get(start..end).unwrap_or(&[]).to_vec()
}

pub fn postfix<T: ToChild + Clone>(
    pattern: &'_ [T],
    index: usize,
) -> Vec<T> {
    pattern.get(index..).unwrap_or(&[]).to_vec()
}

#[track_caller]
#[tracing::instrument(skip(pattern, range, replace))]
pub fn replace_in_pattern(
    mut pattern: impl AsPatternMut,
    range: impl PatternRangeIndex,
    replace: impl IntoPattern,
) -> Pattern {
    pattern
        .borrow_mut()
        .splice(range, replace.into_pattern())
        .collect()
}

pub fn single_child_patterns(
    halves: Vec<Pattern>
) -> Result<Child, Vec<Pattern>> {
    match (halves.len(), halves.first()) {
        (1, Some(first)) =>
            single_child_pattern(first.clone()).map_err(|_| halves),
        _ => Err(halves),
    }
}

pub fn single_child_pattern(half: Pattern) -> Result<Child, Pattern> {
    match (half.len(), half.first()) {
        (1, Some(first)) => Ok(*first),
        _ => Err(half),
    }
}

/// Split a pattern before the specified index
pub fn split_pattern_at_index<T: ToChild + Clone>(
    pattern: &'_ [T],
    index: usize,
) -> (Vec<T>, Vec<T>) {
    (prefix(pattern, index), postfix(pattern, index))
}

pub fn split_context<T: ToChild + Clone>(
    pattern: &'_ [T],
    index: usize,
) -> (Vec<T>, Vec<T>) {
    (prefix(pattern, index), postfix(pattern, index + 1))
}

pub fn double_split_context(
    pattern: PatternView<'_>,
    left_index: usize,
    right_index: usize,
) -> (Pattern, Pattern, Pattern) {
    let (prefix, rem) = split_context(pattern, left_index);
    if left_index < right_index {
        let (infix, postfix) =
            split_context(&rem, right_index - (left_index + 1));
        (prefix, infix, postfix)
    } else {
        (prefix, vec![], rem)
    }
}
