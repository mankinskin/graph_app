use crate::*;

#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub struct SearchPath {
    pub start: ChildPath,
    pub end: ChildPath,
}
//impl From<ChildPath> for SearchPath {
//    fn from(start: ChildPath) -> Self {
//        let entry = start.child_location();
//        Self {
//            start,
//            end: ChildPath::Leaf(PathLeaf::new({
//                entry,
//                width: 0,
//            })),
//        }
//    }
//}
impl<> SearchPath {
    #[allow(unused)]
    pub fn into_paths(self) -> (ChildPath, ChildPath) {
        (
            self.start,
            self.end
        )
    }
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
//impl RangePath for SearchPath {
//}

impl AdvanceExit for SearchPath {
    fn is_finished<
        'a: 'g,
        'g,
        T: Tokenize,
        Trav: Traversable<T>,
    >(&self, trav: &'a Trav) -> bool {
        let location = <Self as GraphChild<End>>::child_location(self);
        let pattern = trav.graph().expect_pattern_at(&location);
        self.is_pattern_finished(pattern)
    }
    fn next_exit_pos<
        'a: 'g,
        'g,
        T: Tokenize,
        D: MatchDirection,
        Trav: Traversable<T>,
    >(&self, trav: &'a Trav) -> Result<Option<usize>, ()> {
        let location = self.get_path_child_location();
        let pattern = trav.graph().expect_pattern_at(&location);
        self.pattern_next_exit_pos::<D, _>(pattern)
    }
}
impl Wide for SearchPath {
    fn width(&self) -> usize {
        self.start.width()
    }
}
impl WideMut for SearchPath {
    fn width_mut(&mut self) -> &mut usize {
        self.start.width_mut()
    }
}

impl PartialOrd for SearchPath {
    fn partial_cmp(&self, other: &SearchPath) -> Option<Ordering> {
        match self.width().cmp(&other.width()) {
            Ordering::Equal =>
                match (self.min_path_segments(), other.min_path_segments()) {
                    (1, 2..) => Some(Ordering::Greater),
                    (2.., 1) => Some(Ordering::Less),
                    _ =>
                        HasMatchPaths::num_path_segments(self).partial_cmp(
                            &HasMatchPaths::num_path_segments(other)
                        ).map(Ordering::reverse),
                },
            o => Some(o),
        }
    }
}