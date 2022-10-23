use crate::*;
use super::*;

pub(crate) trait PathReduce: Sized {
    fn into_reduced<
        'a: 'g,
        'g,
        T: Tokenize,
        D: MatchDirection,
        Trav: Traversable<'a, 'g, T>,
    >(self, trav: &'a Trav) -> Self;
    fn reduce<
        'a: 'g,
        'g,
        T: Tokenize,
        D: MatchDirection,
        Trav: Traversable<'a, 'g, T>,
    >(&mut self, trav: &'a Trav) {
        replace_with::replace_with_or_abort(
            self,
            |self_| self_.into_reduced::<_, D, _>(trav)
        );
    }
}