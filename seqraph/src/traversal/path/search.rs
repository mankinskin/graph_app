use crate::*;

#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub struct SearchPath {
    pub start: StartPath,
    pub end: EndPath,
}
impl From<StartPath> for SearchPath {
    fn from(start: StartPath) -> Self {
        let entry = start.entry();
        Self {
            start,
            end: EndPath {
                entry,
                path: vec![],
                width: 0,
            },
        }
    }
}
impl<> SearchPath {
    #[allow(unused)]
    pub fn into_paths(self) -> (StartPath, EndPath) {
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
    //    self.start.reduce::<_, D, _>(&*graph);
    //    FoundPath::new::<_, D, _>(&*graph, self)
    //}
    //pub fn reduce<
    //    T: Tokenize,
    //    D: MatchDirection,
    //    Trav: Traversable<T>,
    //>(mut self, trav: Trav) -> FoundPath {
    //    let graph = trav.graph();
    //    self.start.reduce::<_, D, _>(&*graph);
    //    self.end.reduce::<_, D, _>(&*graph);
    //    FoundPath::new::<_, D, _>(&*graph, self)
    //}

}
//impl RangePath for SearchPath {
//}
impl HasMatchPaths for SearchPath {
    fn into_paths(self) -> (StartPath, EndPath) {
        (self.start, self.end)
    }
}
impl PathRoot for SearchPath {
    fn root(&self) -> ChildLocation {
        self.entry()
    }
}
impl GraphEntry for SearchPath {
    fn entry(&self) -> ChildLocation {
        self.start.entry()
    }
}
impl HasStartPath for SearchPath {
    fn start_path(&self) -> &[ChildLocation] {
        self.start.start_path()
    }
}
impl GraphStart for SearchPath {}
impl GraphExit for SearchPath {
    fn get_exit_location(&self) -> ChildLocation {
        self.end.entry
    }
}
impl HasEndPath for SearchPath {
    fn end_path(&self) -> &[ChildLocation] {
        self.end.end_path()
    }
}

impl AdvanceExit for SearchPath {
    fn is_finished<
        'a: 'g,
        'g,
        T: Tokenize,
        Trav: Traversable<T>,
    >(&self, trav: &'a Trav) -> bool {
        let location = self.get_exit_location();
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
        let location = self.get_end_location();
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