use std::borrow::Borrow;
use crate::*;

#[derive(Debug, Clone, Eq)]
pub(crate) struct SearchPath {
    pub(crate) start: StartPath,
    pub(crate) inner_width: usize,
    pub(crate) end: EndPath,
}

impl From<StartPath> for SearchPath {
    fn from(start: StartPath) -> Self {
        Self::new(start)
    }
}
impl<'a: 'g, 'g> SearchPath {
    pub fn new(start: StartPath) -> Self {
        Self {
            start,
            inner_width: 0,
            end: EndPath {
                entry: start.entry(),
                width: 0,
                path: vec![],
            },
        }
    }
    pub fn into_paths(self) -> (StartPath, EndPath) {
        (
            self.start,
            self.end
        )
    }
    pub(crate) fn is_complete<
        T: Tokenize,
        D: MatchDirection,
        Trav: Traversable<'a, 'g, T>,
    >(&self, trav: &'a Trav) -> bool {
        let pattern = self.start.pattern(trav);
        DirectedBorderPath::<D>::pattern_is_complete(&self.start, &pattern[..]) &&
            self.end.path.is_empty() &&
            <EndPath as DirectedBorderPath<D>>::pattern_entry_outer_pos(pattern, self.get_exit_pos()).is_none()
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
impl TraversalPath for SearchPath {
    fn reduce_end<
        'a: 'g,
        'g,
        T: Tokenize,
        D: MatchDirection,
        Trav: Traversable<'a, 'g, T>,
    >(mut self, trav: &'a Trav) -> SearchFoundPath {
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
            TraversalPath::move_width_into_start(&mut self);
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
impl GraphEntry for SearchPath {
    fn get_entry_location(&self) -> ChildLocation {
        self.start.get_entry_location()
    }
}
impl HasStartPath for SearchPath {
    fn get_start_path(&self) -> &[ChildLocation] {
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
    fn get_end_path(&self) -> &[ChildLocation] {
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
impl Wide for SearchPath {
    fn width(&self) -> usize {
        self.start.width() + self.inner_width + self.end.width()
    }
}
impl AdvanceablePath for SearchPath {}