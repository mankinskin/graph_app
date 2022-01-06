use super::*;
use std::{
    borrow::Borrow,
    ops::RangeBounds,
    slice::SliceIndex,
};
pub type Pattern = Vec<Child>;
pub type PatternView<'a> = &'a [Child];
pub trait PatternRangeIndex:
    SliceIndex<[Child], Output = [Child]> + RangeBounds<usize> + Iterator<Item = usize> + Debug
{
}
impl<
        T: SliceIndex<[Child], Output = [Child]> + RangeBounds<usize> + Iterator<Item = usize> + Debug,
    > PatternRangeIndex for T
{
}
/// trait for types which can be converted to a pattern with a known size
pub trait IntoPattern: IntoIterator + Sized
where
    <Self as IntoIterator>::Item: AsChild,
{
    type Token: AsChild;
    fn into_pattern(self) -> Pattern {
        self.into_iter().map(|x| x.as_child()).collect()
    }
    fn as_pattern_view(&'_ self) -> &'_ [Self::Token];
    fn is_empty(&self) -> bool;
}
impl<T: AsChild> IntoPattern for Vec<T> {
    type Token = T;
    fn as_pattern_view(&'_ self) -> &'_ [Self::Token] {
        self.as_slice()
    }
    fn is_empty(&self) -> bool {
        self.is_empty()
    }
}
impl<'a, T: 'a> IntoPattern for &'a Vec<T>
where
    &'a T: AsChild,
    T: AsChild,
{
    type Token = T;
    fn as_pattern_view(&'_ self) -> &'_ [Self::Token] {
        self.as_slice()
    }
    fn is_empty(&self) -> bool {
        (*self).is_empty()
    }
}
impl<'a, T: 'a> IntoPattern for &'a mut Vec<T>
where
    &'a mut T: AsChild,
    T: AsChild,
{
    type Token = T;
    fn as_pattern_view(&'_ self) -> &'_ [Self::Token] {
        self.as_slice()
    }
    fn is_empty(&self) -> bool {
        (**self).is_empty()
    }
}
impl<'a, T: 'a> IntoPattern for &'a [T]
where
    &'a T: AsChild,
    T: AsChild,
{
    type Token = T;
    fn as_pattern_view(&'_ self) -> &'_ [Self::Token] {
        self
    }
    fn is_empty(&self) -> bool {
        (*self).is_empty()
    }
}
impl IntoPattern for Child {
    type Token = Child;
    fn as_pattern_view(&'_ self) -> &'_ [Self::Token] {
        std::slice::from_ref(self)
    }
    fn is_empty(&self) -> bool {
        false
    }
}

pub fn pattern_width<T: Borrow<Child>>(pat: impl IntoIterator<Item = T>) -> TokenPosition {
    pat.into_iter().map(|c| c.borrow().width()).sum()
}
pub fn prefix<T: AsChild + Clone>(
    pattern: &'_ [T],
    index: PatternId,
) -> Vec<T> {
    pattern.get(..index).unwrap_or(pattern).to_vec()
}
pub fn infix<T: AsChild + Clone>(
    pattern: &'_ [T],
    start: PatternId,
    end: PatternId,
) -> Vec<T> {
    pattern.get(start..end).unwrap_or(&[]).to_vec()
}
pub fn postfix<T: AsChild + Clone>(
    pattern: &'_ [T],
    index: PatternId,
) -> Vec<T> {
    pattern.get(index..).unwrap_or(&[]).to_vec()
}
pub fn replace_in_pattern(
    pattern: impl IntoPattern<Item = impl AsChild>,
    range: impl PatternRangeIndex,
    replace: impl IntoPattern<Item = impl AsChild>,
) -> Pattern {
    let mut pattern: Pattern = pattern.into_pattern();
    pattern.splice(range, replace.into_pattern());
    pattern
}
pub fn single_child_patterns(halves: Vec<Pattern>) -> Result<Child, Vec<Pattern>> {
    match (halves.len(), halves.first()) {
        (1, Some(first)) => single_child_pattern(first.clone()).map_err(|_| halves),
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
pub fn split_pattern_at_index<T: AsChild + Clone>(
    pattern: &'_ [T],
    index: PatternId,
) -> (Vec<T>, Vec<T>) {
    (prefix(pattern, index), postfix(pattern, index))
}
pub fn split_context<T: AsChild + Clone>(
    pattern: &'_ [T],
    index: PatternId,
) -> (Vec<T>, Vec<T>) {
    (prefix(pattern, index), postfix(pattern, index + 1))
}
pub fn double_split_context(
    pattern: PatternView<'_>,
    left_index: PatternId,
    right_index: PatternId,
) -> (Pattern, Pattern, Pattern) {
    let (prefix, rem) = split_context(pattern, left_index);
    if left_index < right_index {
        let (infix, postfix) = split_context(&rem, right_index - (left_index + 1));
        (prefix, infix, postfix)
    } else {
        (prefix, vec![], rem)
    }
}
