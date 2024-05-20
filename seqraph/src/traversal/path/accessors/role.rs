use crate::{
    traversal::path::accessors::border::PathBorder,
    vertex::{
        location::ChildLocation,
        pattern::{
            postfix,
            prefix,
        },
        PatternId,
    },
};
use std::{
    borrow::Borrow,
    fmt::Debug,
};

#[derive(Hash, Debug, Clone, Eq, PartialEq, Default)]
pub struct Start;

#[derive(Hash, Debug, Clone, Eq, PartialEq, Default)]
pub struct End;

pub trait PathRole: 'static + Debug + PathBorder + Default {
    type TopDownPathIter<I: Borrow<ChildLocation>, T: DoubleEndedIterator<Item=I> + ExactSizeIterator>: DoubleEndedIterator<Item=I> + ExactSizeIterator;
    fn top_down_iter<
        I: Borrow<ChildLocation>,
        T: DoubleEndedIterator<Item = I> + ExactSizeIterator,
    >(
        collection: T
    ) -> Self::TopDownPathIter<I, T>;
    fn bottom_up_iter<
        I: Borrow<ChildLocation>,
        T: DoubleEndedIterator<Item = I> + ExactSizeIterator,
    >(
        collection: T
    ) -> std::iter::Rev<Self::TopDownPathIter<I, T>> {
        Self::top_down_iter(collection).rev()
    }
    /// get remaining pattern agains matching direction excluding index
    fn back_context<T: crate::vertex::indexed::AsChild + Clone>(
        pattern: &'_ [T],
        index: PatternId,
    ) -> Vec<T>;
    fn normalize_index<T: crate::vertex::indexed::AsChild>(
        pattern: &'_ [T],
        index: usize,
    ) -> usize;
    fn split_end<T: crate::vertex::indexed::AsChild + Clone>(
        pattern: &'_ [T],
        index: PatternId,
    ) -> Vec<T>;
    fn split_end_normalized<T: crate::vertex::indexed::AsChild + Clone>(
        pattern: &'_ [T],
        index: PatternId,
    ) -> Vec<T> {
        Self::split_end(pattern, Self::normalize_index(pattern, index))
    }
    fn directed_pattern_split<T: crate::vertex::indexed::AsChild + Clone>(
        pattern: &'_ [T],
        index: usize,
    ) -> (Vec<T>, Vec<T>) {
        (
            Self::back_context(pattern, index),
            Self::split_end_normalized(pattern, index),
        )
    }
}

impl PathRole for Start {
    type TopDownPathIter<
        I: Borrow<ChildLocation>,
        T: DoubleEndedIterator<Item = I> + ExactSizeIterator,
    > = std::iter::Rev<T>;
    fn top_down_iter<
        I: Borrow<ChildLocation>,
        T: DoubleEndedIterator<Item = I> + ExactSizeIterator,
    >(
        collection: T
    ) -> Self::TopDownPathIter<I, T> {
        collection.rev()
    }
    fn back_context<T: crate::vertex::indexed::AsChild + Clone>(
        pattern: &'_ [T],
        index: PatternId,
    ) -> Vec<T> {
        prefix(pattern, index)
    }
    fn normalize_index<T: crate::vertex::indexed::AsChild>(
        _pattern: &'_ [T],
        index: usize,
    ) -> usize {
        index
    }
    fn split_end<T: crate::vertex::indexed::AsChild + Clone>(
        pattern: &'_ [T],
        index: PatternId,
    ) -> Vec<T> {
        postfix(pattern, index)
    }
}
impl PathRole for End {
    type TopDownPathIter<
        I: Borrow<ChildLocation>,
        T: DoubleEndedIterator<Item = I> + ExactSizeIterator,
    > = T;
    fn top_down_iter<
        I: Borrow<ChildLocation>,
        T: DoubleEndedIterator<Item = I> + ExactSizeIterator,
    >(
        collection: T
    ) -> Self::TopDownPathIter<I, T> {
        collection
    }
    fn back_context<T: crate::vertex::indexed::AsChild + Clone>(
        pattern: &'_ [T],
        index: PatternId,
    ) -> Vec<T> {
        postfix(pattern, index + 1)
    }
    fn normalize_index<T: crate::vertex::indexed::AsChild>(
        pattern: &'_ [T],
        index: usize,
    ) -> usize {
        pattern.len() - index - 1
    }
    fn split_end<T: crate::vertex::indexed::AsChild + Clone>(
        pattern: &'_ [T],
        index: PatternId,
    ) -> Vec<T> {
        prefix(pattern, index + 1)
    }
}
