use crate::*;

#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub(crate) struct SearchPath {
    pub(crate) start: StartPath,
    pub(crate) end: EndPath,
}
impl From<StartPath> for SearchPath {
    fn from(start: StartPath) -> Self {
        Self::new(start)
    }
}
impl<'a: 'g, 'g> SearchPath {
    pub fn new(start: StartPath) -> Self {
        let entry = start.entry();
        Self {
            start,
            end: EndPath {
                entry,
                path: vec![],
            },
        }
    }
    pub fn new_advanced<
        T: Tokenize,
        D: MatchDirection,
        Trav: Traversable<'a, 'g, T>
    >(
        trav: &'a Trav,
        start: StartPath,
    ) -> Result<Self, StartPath> {
        let mut new = Self::new(start);
        match new.advance_exit_pos::<_, D, _>(trav) {
            Ok(()) => Ok(new),
            Err(()) => Err(new.start)
        }
    }
    #[allow(unused)]
    pub fn into_paths(self) -> (StartPath, EndPath) {
        (
            self.start,
            self.end
        )
    }
    pub fn prev_exit_pos<
        T: Tokenize,
        D: MatchDirection,
        Trav: Traversable<'a, 'g, T>,
    >(&self, trav: &'a Trav) -> Option<usize> {
        let location = self.get_end_location();
        let pattern = trav.graph().expect_pattern_at(&location);
        D::pattern_index_prev(pattern, location.sub_index)
    }
    pub fn reduce_mismatch<
        T: Tokenize,
        D: MatchDirection,
        Trav: Traversable<'a, 'g, T>,
    >(mut self, trav: &'a Trav) -> FoundPath {
        let graph = trav.graph();
        // remove segments pointing to mismatch at pattern head
        while let Some(mut location) = self.end_path_mut().pop() {
            let pattern = graph.expect_pattern_at(&location);
            // skip segments at end of pattern
            if let Some(prev) = D::pattern_index_prev(pattern.borrow(), location.sub_index) {
                location.sub_index = prev;
                self.end_path_mut().push(location);
                break;
            }
        }
        if self.end_path_mut().is_empty() && {
            *self.exit_mut() = self.prev_exit_pos::<_, D, _>(trav).unwrap();
            self.get_entry_pos() == self.get_exit_pos()
        } {
            self.start.pop_path::<_, D, _>(&*graph).into()
        } else {
            FoundPath::new::<_, D, _>(&*graph, self)
        }
    }
    pub fn reduce_end<
        T: Tokenize,
        D: MatchDirection,
        Trav: Traversable<'a, 'g, T>,
    >(mut self, trav: &'a Trav) -> FoundPath {
        let graph = trav.graph();
        // remove segments pointing to mismatch at pattern head
        while let Some(location) = self.end_path_mut().pop() {
            let pattern = graph.expect_pattern_at(&location);
            // skip segments at end of pattern
            if D::pattern_index_next(pattern.borrow(), location.sub_index).is_some() {
                self.end_path_mut().push(location);
                break;
            }
        }
        FoundPath::new::<_, D, _>(&*graph, self)
    }

    pub fn add_match_width<
        T: Tokenize,
        D: MatchDirection,
        Trav: Traversable<'a, 'g, T>,
    >(&mut self, trav: &'a Trav) {
        if let Some(end) = self.get_end::<_, D, _>(trav) {
            let wmut = self.width_mut();
            *wmut += end.width;
        }
    }
}
impl HasStartMatchPath for SearchPath {
    fn start_match_path(&self) -> &StartPath {
        &self.start
    }
    fn start_match_path_mut(&mut self) -> &mut StartPath {
        &mut self.start
    }
}
impl HasEndMatchPath for SearchPath {
    fn end_match_path(&self) -> &EndPath {
        &self.end
    }
    fn end_match_path_mut(&mut self) -> &mut EndPath {
        &mut self.end
    }
}
impl HasMatchPaths for SearchPath {
    fn into_paths(self) -> (StartPath, EndPath) {
        (self.start, self.end)
    }
}
impl GraphEntry for SearchPath {
    fn get_entry_location(&self) -> ChildLocation {
        self.start.get_entry_location()
    }
}
impl HasStartPath for SearchPath {
    fn start_path(&self) -> &[ChildLocation] {
        self.start.path()
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
        self.end.path()
    }
}
impl GraphEnd for SearchPath {}
impl EndPathMut for SearchPath {
    fn end_path_mut(&mut self) -> &mut ChildPath {
        &mut self.end.path
    }
}
impl ExitMut for SearchPath {
    fn exit_mut(&mut self) -> &mut usize {
        self.end.exit_mut()
    }
}
impl End for SearchPath {
    fn get_end<
        'a: 'g,
        'g,
        T: Tokenize,
        D: MatchDirection,
        Trav: Traversable<'a, 'g, T>,
    >(&self, trav: &'a Trav) -> Option<Child> {
        self.get_graph_end(trav)
    }
}
//impl TraversalPath for SearchPath {
//}
impl AdvanceableExit for SearchPath {
    fn is_finished<
        'a: 'g,
        'g,
        T: Tokenize,
        Trav: Traversable<'a, 'g, T>,
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
        Trav: Traversable<'a, 'g, T>,
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
impl AdvanceablePath for SearchPath {}

impl PartialOrd for SearchPath {
    fn partial_cmp(&self, other: &SearchPath) -> Option<Ordering> {
        self.width().partial_cmp(&other.width()).or_else(||
            self.num_path_segments().partial_cmp(
                &other.num_path_segments()
            ).map(Ordering::reverse)
        )
    }
}