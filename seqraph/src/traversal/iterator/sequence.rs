use super::*;

pub struct PathIter<
    'a: 'g,
    'g,
    P: Advance,
    T: Tokenize,
    D: MatchDirection,
    Trav: Traversable<T>,
> {
    path: P,
    trav: Trav,
    _ty: std::marker::PhantomData<(&'g T, D)>
}
//pub trait SequenceIterator: Sized {
//    type Item;
//    fn next(self) -> Result<Self::Item, Self::Item>;
//}
//impl<
//    'a: 'g,
//    'g,
//    P: Advance,
//    T: Tokenize,
//    D: MatchDirection,
//    Trav: Traversable<T>,
//> SequenceIterator for PathIter<P, T, D, Trav> {
//    type Item = P;
//    fn next(mut self) -> Result<Self::Item, Self::Item> {
//        if self.path.advance_next::<_, D, _>(self.trav) {
//            Ok(self.path)
//        } else {
//            Err(self.path)
//        }
//    }
//}
//pub trait IntoSequenceIterator<
//    'a: 'g,
//    'g,
//    T: Tokenize,
//    D: MatchDirection,
//    Trav: Traversable<T>,
//> {
//    type Iter: SequenceIterator;
//    fn into_seq_iter(self, trav: Trav) -> Self::Iter;
//}
//impl<
//    'a: 'g,
//    'g,
//    P: Advance,
//    T: Tokenize,
//    D: MatchDirection,
//    Trav: Traversable<T>,
//> IntoSequenceIterator<T, D, Trav> for P {
//    type Iter = PathIter<P, T, D, Trav>;
//    fn into_seq_iter(self, trav: Trav) -> Self::Iter {
//        PathIter {
//            path: self,
//            trav,
//            _ty: Default::default(),
//        }
//    }
//}