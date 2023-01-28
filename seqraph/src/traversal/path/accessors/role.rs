use crate::*;
#[derive(Hash, Debug, Clone, Eq, PartialEq)]
pub struct Start;

#[derive(Hash, Debug, Clone, Eq, PartialEq)]
pub struct End;

pub trait PathRole: 'static + Debug {
    type TopDownPathIter<I: Borrow<ChildLocation>, T: DoubleEndedIterator<Item=I> + ExactSizeIterator>: DoubleEndedIterator<Item=I> + ExactSizeIterator;
    fn top_down_iter<I: Borrow<ChildLocation>, T: DoubleEndedIterator<Item=I> + ExactSizeIterator>(collection: T) -> Self::TopDownPathIter<I, T>;
    fn bottom_up_iter<I: Borrow<ChildLocation>, T: DoubleEndedIterator<Item=I> + ExactSizeIterator>(collection: T) -> std::iter::Rev<Self::TopDownPathIter<I, T>> {
        Self::top_down_iter(collection).rev()
    }
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