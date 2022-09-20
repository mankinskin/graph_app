use crate::*;
use super::*;

pub(crate) trait PathReduce: Sized {
    fn reduce<
        'a: 'g,
        'g,
        T: Tokenize,
        D: MatchDirection,
        Trav: Traversable<'a, 'g, T>,
    >(&mut self, trav: &'a Trav);
    fn into_reduced<
        'a: 'g,
        'g,
        T: Tokenize,
        D: MatchDirection,
        Trav: Traversable<'a, 'g, T>,
    >(mut self, trav: &'a Trav) -> Self {
        self.reduce::<_, D, _>(trav);
        self
    }
}