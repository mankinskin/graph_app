use crate::*;
use super::*;


pub trait PathSimplify: Sized + PathComplete {
    fn into_simplified<
        T: Tokenize,
        D: MatchDirection,
        Trav: Traversable<T>,
    >(self, trav: &Trav) -> Self;
    fn simplify<
        T: Tokenize,
        D: MatchDirection,
        Trav: Traversable<T>,
    >(&mut self, trav: &Trav) {
	    unsafe {
	    	let old = std::ptr::read(self);
	    	let new = old.into_simplified::<_, D, _>(trav);
	    	std::ptr::write(self, new);
	    }
    }
}
impl<P: MatchEndPath + PathSimplify> PathSimplify for MatchEnd<P> {
    fn into_simplified<
        T: Tokenize,
        D: MatchDirection,
        Trav: Traversable<T>,
    >(self, trav: &Trav) -> Self {
        if let Some(c) = match self.get_path() {
            Some(p) => p.into_complete(),
            None => None,
        } {
            MatchEnd::Complete(c)
        } else {
            self    
        }
        //if let MatchEnd::Path(path) = self {
        //    path.pop_path::<_, D, _>(trav)
        //} else {
        //    self    
        //}
    }
}
impl<R: PathRole> PathSimplify for RolePath<R> {
    fn into_simplified<
        T: Tokenize,
        D: MatchDirection,
        Trav: Traversable<T>,
    >(mut self, trav: &Trav) -> Self {
        let graph = trav.graph();
        // remove segments pointing to mismatch at pattern head
        while let Some(location) = self.path_mut().pop() {
            let pattern = graph.expect_pattern_at(&location);
            // skip segments at end of pattern
            if D::pattern_index_next(pattern.borrow(), location.sub_index).is_some() {
                self.path_mut().push(location);
                break;
            }
        }
        self
    }
}
//impl<P: PathSimplify> PathSimplify for OriginPath<P> {
//    fn into_simplified<
//        T: Tokenize,
//        D: MatchDirection,
//        Trav: Traversable<T>,
//    >(mut self, trav: &Trav) -> Self {
//        self.postfix.simplify::<_, D, _>(trav);
//        self
//    }
//}
