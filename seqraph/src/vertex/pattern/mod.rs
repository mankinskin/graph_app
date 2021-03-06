mod pattern_stream;
mod pattern_location;
mod pattern_range;

use super::{
    *,
};
use crate::*;
pub use {
    pattern_range::*,
    pattern_location::*,
    pattern_stream::*,
};
pub type Pattern = Vec<Child>;
pub type PatternView<'a> = &'a [Child];

/// trait for types which can be converted to a pattern with a known size
pub trait IntoPattern:
    IntoIterator<Item = Self::Elem, IntoIter = Self::Iter>
    + Sized
    + Borrow<[Child]>
    + Debug
{
    type Iter: ExactSizeIterator + DoubleEndedIterator<Item=Self::Elem>;
    type Elem: AsChild;

    fn into_pattern(self) -> Pattern {
        self.into_iter().map(|x| x.as_child()).collect()
    }
    fn is_empty(&self) -> bool {
        self.borrow().is_empty()
    }
}
impl<C, It, T> IntoPattern for T
    where
        C: AsChild,
        It: DoubleEndedIterator<Item=C> + ExactSizeIterator,
        T: IntoIterator<Item=C, IntoIter=It> + Borrow<[Child]> + Debug,
{
    type Iter = It;
    type Elem = C;
}
/// trait for types which can be converted to a pattern with a known size
pub trait AsPatternMut: BorrowMut<Vec<Child>> + Debug {
}
impl<T> AsPatternMut for T
    where
        T: BorrowMut<Vec<Child>>
            + Debug
{
}
pub fn pattern_width<T: Borrow<Child>>(pat: impl IntoIterator<Item = T>) -> TokenPosition {
    pat.into_iter().map(|c| c.borrow().width()).sum()
}
pub fn prefix<T: AsChild + Clone>(
    pattern: &'_ [T],
    index: PatternId,
) -> Vec<T> {
    pattern.get(0..index).unwrap_or(pattern).to_vec()
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
#[track_caller]
#[tracing::instrument]
pub fn replace_in_pattern(
    mut pattern: impl AsPatternMut ,
    range: impl PatternRangeIndex,
    replace: impl IntoPattern,
) -> Pattern {
    //print!("replacing {:?} in {:?}[{:#?}]",
    //    replace.borrow().iter().map(|c| c.index()).collect_vec(),
    //    pattern.borrow().iter().map(|c| c.index()).collect_vec(),
    //    range
    //);
    let old = pattern.borrow_mut().splice(range, replace.into_pattern()).collect();
    //println!(" -> {:?}", pattern.borrow().iter().map(|c| c.index()).collect_vec());
    old
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
