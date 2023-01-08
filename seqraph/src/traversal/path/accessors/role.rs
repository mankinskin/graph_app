use crate::*;
#[derive(Hash, Debug, Clone, Eq, PartialEq)]
pub struct Start;

#[derive(Hash, Debug, Clone, Eq, PartialEq)]
pub struct End;

pub trait PathRole {
    type TopDownPathIter<I: Borrow<ChildLocation>, T: DoubleEndedIterator<Item=I> + ExactSizeIterator>: Iterator<Item=I> + ExactSizeIterator;
    fn top_down_iter<I: Borrow<ChildLocation>, T: DoubleEndedIterator<Item=I> + ExactSizeIterator>(collection: T) -> Self::TopDownPathIter<I, T>;
}

impl PathRole for Start {
    type TopDownPathIter<I: Borrow<ChildLocation>, T: DoubleEndedIterator<Item=I> + ExactSizeIterator> = std::iter::Rev<T>;
    fn top_down_iter<I: Borrow<ChildLocation>, T: DoubleEndedIterator<Item=I> + ExactSizeIterator>(collection: T) -> Self::TopDownPathIter<I, T> {
        collection.rev()
    }
}
impl PathRole for End {
    type TopDownPathIter<I: Borrow<ChildLocation>, T: DoubleEndedIterator<Item=I> + ExactSizeIterator> = T;
    fn top_down_iter<I: Borrow<ChildLocation>, T: DoubleEndedIterator<Item=I> + ExactSizeIterator>(collection: T) -> Self::TopDownPathIter<I, T> {
        collection
    }
}