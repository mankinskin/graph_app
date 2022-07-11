use super::*;

pub(crate) type IndexingFoundPath = FoundPath<IndexingPath>;

#[derive(Debug, Clone)]
pub(crate) struct IndexingPath {
    pub(crate) start: StartLeaf,
    pub(crate) inner_width: usize,
    pub(crate) end: EndPath
}

impl From<StartLeaf> for IndexingPath {
    fn from(start: StartLeaf) -> Self {
        Self::new(start)
    }
}
impl<'a: 'g, 'g> IndexingPath {
    pub fn new(start: StartLeaf) -> Self {
        let entry = start.entry();
        Self {
            start,
            inner_width: 0,
            end: EndPath {
                entry,
                width: 0,
                path: vec![],
            }
        }
    }
    pub fn into_start_path(self) -> StartPath {
        StartPath::Leaf(self.start)
    }
    pub(crate) fn next_pos<
        T: Tokenize,
        D: MatchDirection,
        Trav: Traversable<'a, 'g, T>,
    >(&self, trav: &'a Trav) -> Option<usize> {
        let location = self.get_end_location();
        let pattern = trav.graph().expect_pattern_at(&location);
        D::pattern_index_next(pattern, self.get_exit_pos())
    }
}
impl HasStartMatchPath for IndexingPath {
    fn get_start_match_path(&self) -> StartPath {
        StartPath::Leaf(self.start.clone())
    }
}
impl HasEndMatchPath for IndexingPath {
    fn get_end_match_path(&self) -> EndPath {
        self.end.clone()
    }
}
impl HasMatchPaths for IndexingPath {
    fn into_paths(self) -> (StartPath, EndPath) {
        (StartPath::Leaf(self.start), self.end)
    }
}
impl TraversalPath for IndexingPath {
    fn reduce_end<
        'a: 'g,
        'g,
        T: Tokenize,
        D: MatchDirection,
        Trav: Traversable<'a, 'g, T>,
    >(mut self, trav: &'a Trav) -> IndexingFoundPath {
        let graph = trav.graph();
        //self.reduce_end_path::<T, D>(&*graph);
        // remove segments pointing to mismatch at pattern head
        while let Some(location) = self.end.path.pop() {
            let pattern = graph.expect_pattern_at(&location);
            // skip segments at end of pattern
            if D::pattern_index_next(pattern.borrow(), location.sub_index).is_some() {
                self.end.path.push(location);
                break;
            }
        }
        if self.end.path.is_empty() {
            self.move_width_into_start();
        }
        FoundPath::new::<_, D, _>(trav, self)
    }
    fn move_width_into_start(&mut self) {
        *self.start.width_mut() += self.inner_width + self.end.width();
        self.inner_width = 0;
        *self.end.width_mut() = 0;
    }
    fn on_match<
        'a: 'g,
        'g,
        T: Tokenize,
        D: MatchDirection,
        Trav: Traversable<'a, 'g, T>,
    >(&mut self, trav: &'a Trav) {
        let width = self.get_end::<_, D, _>(trav).width;
        let wmut = if self.end.path().is_empty() {
            &mut self.inner_width
        } else {
            self.end.width_mut()
        };
        *wmut += width;
    }
}
impl GraphEntry for IndexingPath {
    fn get_entry_location(&self) -> ChildLocation {
        self.start.get_entry_location()
    }
}
impl HasStartPath for IndexingPath {
    fn get_start_path(&self) -> &[ChildLocation] {
        self.start.get_start_path()
    }
}
impl GraphStart for IndexingPath {}
impl GraphExit for IndexingPath {
    fn get_exit_location(&self) -> ChildLocation {
        self.end.entry
    }
}
impl HasEndPath for IndexingPath {
    fn get_end_path(&self) -> &[ChildLocation] {
        self.end.path()
    }
}
impl GraphEnd for IndexingPath {}
impl EndPathMut for IndexingPath {
    fn end_path_mut(&mut self) -> &mut ChildPath {
        &mut self.end.path
    }
}
impl ExitMut for IndexingPath {
    fn exit_mut(&mut self) -> &mut usize {
        self.end.exit_mut()
    }
}
impl End for IndexingPath {
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

impl PathFinished for IndexingPath {
    fn is_finished(&self) -> bool {
        false
    }
    fn set_finished(&mut self) {
    }
}
impl ReduciblePath for IndexingPath {
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
impl AdvanceableExit for IndexingPath {
    fn next_exit_pos<
        'a: 'g,
        'g,
        T: Tokenize,
        D: MatchDirection,
        Trav: Traversable<'a, 'g, T>,
    >(&self, trav: &'a Trav) -> Option<usize> {
        self.next_pos::<_, D, _>(trav)
    }
}
impl Wide for IndexingPath {
    fn width(&self) -> usize {
        self.start.width() + self.inner_width + self.end.width()
    }
}
impl AdvanceablePath for IndexingPath {}

impl ResultOrd for IndexingPath {
    fn is_complete(&self) -> bool {
        false
    }
}