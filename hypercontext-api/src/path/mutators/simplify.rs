use crate::traversal::traversable::Traversable;
pub trait PathSimplify: Sized {
    fn into_simplified<Trav: Traversable>(
        self,
        trav: &Trav,
    ) -> Self;
    fn simplify<Trav: Traversable>(
        &mut self,
        trav: &Trav,
    ) {
        unsafe {
            let old = std::ptr::read(self);
            let new = old.into_simplified(trav);
            std::ptr::write(self, new);
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
//        D: ,
//        Trav: Traversable<T>,
//    >(mut self, trav: &Trav) -> Self {
//        self.postfix.simplify::<_, D, _>(trav);
//        self
//    }
//}
