use super::*;

pub(crate) struct PathIter<
    'a: 'g,
    'g,
    P: AdvanceablePath,
    T: Tokenize,
    D: MatchDirection,
    Trav: Traversable<'a, 'g, T>,
> {
    path: P,
    trav: &'a Trav,
    _ty: std::marker::PhantomData<(&'g T, D)>
}
//pub(crate) trait SequenceIterator: Sized {
//    type Item;
//    fn next(self) -> Result<Self::Item, Self::Item>;
//}
//impl<
//    'a: 'g,
//    'g,
//    P: AdvanceablePath,
//    T: Tokenize,
//    D: MatchDirection,
//    Trav: Traversable<'a, 'g, T>,
//> SequenceIterator for PathIter<'a, 'g, P, T, D, Trav> {
//    type Item = P;
//    fn next(mut self) -> Result<Self::Item, Self::Item> {
//        if self.path.advance_next::<_, D, _>(self.trav) {
//            Ok(self.path)
//        } else {
//            Err(self.path)
//        }
//    }
//}
//pub(crate) trait IntoSequenceIterator<
//    'a: 'g,
//    'g,
//    T: Tokenize,
//    D: MatchDirection,
//    Trav: Traversable<'a, 'g, T>,
//> {
//    type Iter: SequenceIterator;
//    fn into_seq_iter(self, trav: &'a Trav) -> Self::Iter;
//}
//impl<
//    'a: 'g,
//    'g,
//    P: AdvanceablePath,
//    T: Tokenize,
//    D: MatchDirection,
//    Trav: Traversable<'a, 'g, T>,
//> IntoSequenceIterator<'a, 'g, T, D, Trav> for P {
//    type Iter = PathIter<'a, 'g, P, T, D, Trav>;
//    fn into_seq_iter(self, trav: &'a Trav) -> Self::Iter {
//        PathIter {
//            path: self,
//            trav,
//            _ty: Default::default(),
//        }
//    }
//}