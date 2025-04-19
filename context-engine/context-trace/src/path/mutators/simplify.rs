use crate::trace::has_graph::HasGraph;

pub trait PathSimplify: Sized {
    fn into_simplified<G: HasGraph>(
        self,
        trav: &G,
    ) -> Self;
    fn simplify<G: HasGraph>(
        &mut self,
        trav: &G,
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
//        G: HasGraph,
//    >(mut self, trav: &G) -> Self {
//        let graph = trav.graph();
//        // remove segments pointing to mismatch at pattern head
//        while let Some(location) = self.path_mut().pop() {
//            let pattern = graph.expect_pattern_at(&location);
//            // skip segments at end of pattern
//            if G::Direction::pattern_index_next(pattern.borrow(), location.sub_index).is_some() {
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
//        G: HasGraph<T>,
//    >(mut self, trav: &G) -> Self {
//        self.postfix.simplify::<_, D, _>(trav);
//        self
//    }
//}
