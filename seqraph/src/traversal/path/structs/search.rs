use crate::*;

#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub struct SearchPath {
    pub root: PatternLocation,
    pub start: RolePath<Start>,
    pub end: RolePath<End>,
}
impl SearchPath {
    //#[allow(unused)]
    //pub fn into_paths(self) -> (RolePath<Start>, RolePath<End>) {
    //    (
    //        self.start,
    //        self.end
    //    )
    //}
    //pub fn reduce_start<
    //    T: Tokenize,
    //    D: MatchDirection,
    //    Trav: Traversable<T>,
    //>(mut self, trav: Trav) -> FoundPath {
    //    let graph = trav.graph();
    //    self.start.simplify::<_, D, _>(&*graph);
    //    FoundPath::new::<_, D, _>(&*graph, self)
    //}
    //pub fn simplify<
    //    T: Tokenize,
    //    D: MatchDirection,
    //    Trav: Traversable<T>,
    //>(mut self, trav: Trav) -> FoundPath {
    //    let graph = trav.graph();
    //    self.start.simplify::<_, D, _>(&*graph);
    //    self.end.simplify::<_, D, _>(&*graph);
    //    FoundPath::new::<_, D, _>(&*graph, self)
    //}

}

//impl Wide for SearchPath {
//    fn width(&self) -> usize {
//        self.start.width()
//    }
//}
//impl WideMut for SearchPath {
//    fn width_mut(&mut self) -> &mut usize {
//        self.start.width_mut()
//    }
//}

//impl PartialOrd for SearchPath {
//    fn partial_cmp(&self, other: &SearchPath) -> Option<Ordering> {
//        match self.width().cmp(&other.width()) {
//            Ordering::Equal =>
//                match (self.min_path_segments(), other.min_path_segments()) {
//                    (1, 2..) => Some(Ordering::Greater),
//                    (2.., 1) => Some(Ordering::Less),
//                    _ =>
//                        HasMatchPaths::num_path_segments(self).partial_cmp(
//                            &HasMatchPaths::num_path_segments(other)
//                        ).map(Ordering::reverse),
//                },
//            o => Some(o),
//        }
//    }
//}