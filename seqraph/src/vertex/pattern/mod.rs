mod pattern_stream;
mod pattern_location;
mod pattern_range;

use super::{
    *,
};
use std::borrow::Borrow;
pub use {
    pattern_range::*,
    pattern_location::*,
    pattern_stream::*,
};
pub type Pattern = Vec<Child>;
pub type PatternView<'a> = &'a [Child];
/// trait for types which can be converted to a pattern with a known size
pub trait IntoPattern: IntoIterator<Item = Self::Elem, IntoIter = Self::Iter> + Sized + Borrow<[Child]> {
    type Iter: ExactSizeIterator + DoubleEndedIterator<Item=Self::Elem>;
    type Elem: AsChild;

    fn into_pattern(self) -> Pattern {
        self.into_iter().map(|x| x.as_child()).collect()
    }
    //fn as_pattern_view(&'_ self) -> &'_ [Child];
    fn is_empty(&self) -> bool;
}
//impl<T: AsChild> IntoPattern for Vec<T> {
//    type Iter = <Vec<T> as IntoIterator>::IntoIter;
//    type Token = T;
//    type Elem = Self::Token;
//    fn as_pattern_view(&'_ self) -> &'_ [Self::Token] {
//        self.as_slice()
//    }
//    fn is_empty(&self) -> bool {
//        self.is_empty()
//    }
//}
//impl<'a, T: 'a> IntoPattern for &'a Vec<T>
//where
//    &'a T: AsChild,
//    T: AsChild,
//{
//    type Iter = <&'a Vec<T> as IntoIterator>::IntoIter;
//    type Elem = &'a Self::Token;
//    type Token = T;
//    fn as_pattern_view(&'_ self) -> &'_ [Self::Token] {
//        self.as_slice()
//    }
//    fn is_empty(&self) -> bool {
//        (*self).is_empty()
//    }
//}
//impl<'a, T: 'a> IntoPattern for &'a mut Vec<T>
//where
//    &'a mut T: AsChild,
//    T: AsChild,
//{
//    type Iter = <&'a mut Vec<T> as IntoIterator>::IntoIter;
//    type Elem = &'a mut Self::Token;
//    type Token = T;
//    fn as_pattern_view(&'_ self) -> &'_ [Self::Token] {
//        self.as_slice()
//    }
//    fn is_empty(&self) -> bool {
//        (**self).is_empty()
//    }
//}
//impl<'a, T: 'a> IntoPattern for &'a [T]
//where
//    &'a T: AsChild,
//    T: AsChild,
//{
//    type Iter = <&'a [T] as IntoIterator>::IntoIter;
//    type Elem = &'a Self::Token;
//    type Token = T;
//    fn as_pattern_view(&'_ self) -> &'_ [Self::Token] {
//        self
//    }
//    fn is_empty(&self) -> bool {
//        (*self).is_empty()
//    }
//}
//impl IntoPattern for Child {
//    type Iter = std::iter::Once<Self::Token>;
//    type Token = Child;
//    type Elem = Self::Token;
//    fn as_pattern_view(&'_ self) -> &'_ [Self::Token] {
//        std::slice::from_ref(self)
//    }
//    fn is_empty(&self) -> bool {
//        false
//    }
//}
impl<C, It, T> IntoPattern for T
    where
        C: AsChild,
        It: DoubleEndedIterator<Item=C> + ExactSizeIterator,
        T: IntoIterator<Item=C, IntoIter=It> + Borrow<[Child]>,
{
    type Iter = It;
    type Elem = C;
    //fn as_pattern_view(&'_ self) -> &'_ [Child] {
    //    self.borrow()
    //}
    fn is_empty(&self) -> bool {
        self.borrow().is_empty()
    }
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
pub fn replace_in_pattern(
    pattern: impl IntoPattern,
    range: impl PatternRangeIndex,
    replace: impl IntoPattern,
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
