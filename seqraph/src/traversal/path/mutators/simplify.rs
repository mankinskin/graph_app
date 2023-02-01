use crate::*;

pub trait PathSimplify: Sized + PathComplete {
    fn into_simplified<
        Trav: Traversable,
    >(self, trav: &Trav) -> Self;
    fn simplify<
        Trav: Traversable,
    >(&mut self, trav: &Trav) {
	    unsafe {
	    	let old = std::ptr::read(self);
	    	let new = old.into_simplified(trav);
	    	std::ptr::write(self, new);
	    }
    }
}
impl<P: MatchEndPath> PathSimplify for MatchEnd<P> {
    fn into_simplified<
        Trav: Traversable,
    >(self, trav: &Trav) -> Self {
        if let Some(c) = match self.get_path() {
            Some(p) => if p.single_path().is_empty() && {
                let location = p.root_child_location();
                let graph = trav.graph();
                let pattern = graph.expect_pattern_at(&location);
                <Trav::Kind as GraphKind>::Direction::pattern_index_prev(pattern.borrow(), location.sub_index).is_none()
            } {
                Some(p.root_parent())
            } else {
                None
            },
            None => None,
        } {
            MatchEnd::Complete(c)
        } else {
            self    
        }
    }
}
//impl<R: PathRole> PathSimplify for RolePath<R> {
//    fn into_simplified<
//        Trav: Traversable,
//    >(mut self, trav: &Trav) -> Self {
//        let graph = trav.graph();
//        // remove segments pointing to mismatch at pattern head
//        while let Some(location) = self.path_mut().pop() {
//            let pattern = graph.expect_pattern_at(&location);
//            // skip segments at end of pattern
//            if Trav::Direction::pattern_index_next(pattern.borrow(), location.sub_index).is_some() {
//                self.path_mut().push(location);
//                break;
//            }
//        }
//        self
//    }
//}
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
