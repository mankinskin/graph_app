use crate::*;
use super::*;


pub(crate) trait PathReduce: Sized + Send + Sync {
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
	    unsafe {
	    	let old = std::ptr::read(self);
	    	let new = old.into_reduced::<_, D, _>(trav);
	    	std::ptr::write(self, new);
	    }
    }
}