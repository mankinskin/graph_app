use crate::*;

#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub(crate) struct SearchPath {
    pub(crate) start: StartPath,
    pub(crate) end: EndPath,
}
impl ResultOrd for SearchPath {
    fn is_complete(&self) -> bool {
        false
    }
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
//impl HasInnerWidth for SearchPath {
//    fn inner_width(&self) -> usize {
//        self.inner_width
//    }
//    fn inner_width_mut(&mut self) -> &mut usize {
//        &mut self.inner_width
//    }
//}
impl HasMatchPaths for SearchPath {
    fn into_paths(self) -> (StartPath, EndPath) {
        (self.start, self.end)
    }
}
impl TraversalPath for SearchPath {
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
    >(&self, trav: &'a Trav) -> Child {
        self.get_graph_end(trav)
    }
}

impl PathFinished for SearchPath {
    fn is_finished(&self) -> bool {
        false
    }
    fn set_finished(&mut self) {
    }
}
impl ReduciblePath for SearchPath {
    fn prev_exit_pos<
        'a: 'g,
        'g,
        T: Tokenize,
        D: MatchDirection,
        Trav: Traversable<'a, 'g, T>,
    >(&self, trav: &'a Trav) -> Option<usize> {
        let location = self.get_end_location();
        let pattern = trav.graph().expect_pattern_at(&location);
        D::pattern_index_prev(pattern, location.sub_index)
    }
}
impl AdvanceableExit for SearchPath {
    fn pattern_next_exit_pos<
        D: MatchDirection,
        P: IntoPattern
    >(&self, pattern: P) -> Option<usize> {
        D::pattern_index_next(pattern, self.get_exit_pos())
    }
    fn next_exit_pos<
        'a: 'g,
        'g,
        T: Tokenize,
        D: MatchDirection,
        Trav: Traversable<'a, 'g, T>,
    >(&self, trav: &'a Trav) -> Option<usize> {
        let location = self.get_end_location();
        let pattern = trav.graph().expect_pattern_at(&location);
        self.pattern_next_exit_pos::<D, _>(pattern)
    }
}
impl Wide for SearchPath {
    fn width(&self) -> usize {
        self.start.width()
        //+ self.inner_width + self.end.width()
    }
}
impl WideMut for SearchPath {
    fn width_mut(&mut self) -> &mut usize {
        self.start.width_mut()
        //+ self.inner_width + self.end.width()
    }
}
impl AdvanceablePath for SearchPath {}