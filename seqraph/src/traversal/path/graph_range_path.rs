use std::borrow::Borrow;
use crate::*;

#[derive(Debug, Clone, Eq)]
pub(crate) struct GraphRangePath {
    pub(crate) start: StartPath,
    pub(crate) inner_width: usize,
    pub(crate) exit: usize,
    pub(crate) end: ChildPath,
    pub(crate) end_width: usize,
}

impl From<StartPath> for GraphRangePath {
    fn from(start: StartPath) -> Self {
        Self::new(start)
    }
}
impl Into<StartPath> for GraphRangePath {
    fn into(self) -> StartPath {
        self.start
    }
}
impl<'a: 'g, 'g> GraphRangePath {
    pub fn new(start: StartPath) -> Self {
        Self {
            exit: start.entry().sub_index,
            start,
            end: vec![],
            end_width: 0,
            inner_width: 0,
        }
    }
    pub fn into_paths(self) -> (StartPath, EndPath) {
        let entry = self.start.entry();
        let mut exit = entry.clone();
        exit.sub_index = self.exit;
        (
            self.start,
            EndPath {
                entry: exit,
                path: self.end,
                width: self.end_width,
            }
        )
    }
    pub(crate) fn is_complete<
        T: Tokenize + 'a,
        D: MatchDirection + 'a,
        Trav: Traversable<'a, 'g, T>,
    >(&self, trav: &'a Trav) -> bool {
        let pattern = self.start.pattern(trav);
        DirectedBorderPath::<D>::pattern_is_complete(&self.start, &pattern[..]) &&
            self.end.is_empty() &&
            <EndPath as DirectedBorderPath<D>>::pattern_entry_outer_pos(pattern, self.exit).is_none()
    }
    pub(crate) fn next_pos<
        T: Tokenize + 'a,
        D: MatchDirection + 'a,
        Trav: Traversable<'a, 'g, T>,
    >(&self, trav: &'a Trav) -> Option<usize> {
        let location = self.get_end_location();
        let pattern = trav.graph().expect_pattern_at(&location);
        D::pattern_index_next(pattern, self.exit)
    }
}
impl TraversalPath for GraphRangePath {
    fn reduce_end<
        'a: 'g,
        'g,
        T: Tokenize + 'a,
        D: MatchDirection + 'a,
        Trav: Traversable<'a, 'g, T>,
    >(mut self, trav: &'a Trav) -> FoundPath {
        let graph = trav.graph();
        //self.reduce_end_path::<T, D>(&*graph);
        // remove segments pointing to mismatch at pattern head
        while let Some(location) = self.end.pop() {
            let pattern = graph.expect_pattern_at(&location);
            // skip segments at end of pattern
            if D::pattern_index_next(pattern.borrow(), location.sub_index).is_some() {
                self.end.push(location);
                break;
            }
        }
        if self.end.is_empty() {
            self.move_width_into_start();
        }
        FoundPath::new::<_, D, _>(trav, self)
    }
    fn move_width_into_start(&mut self) {
        *self.start.width_mut() += self.inner_width + self.end_width;
        self.inner_width = 0;
        self.end_width = 0;
    }
    fn on_match<
        'a: 'g,
        'g,
        T: Tokenize + 'a,
        D: MatchDirection + 'a,
        Trav: Traversable<'a, 'g, T>,
    >(&mut self, trav: &'a Trav) {
        let width = self.get_end::<_, D, _>(trav).width;
        let wmut = if self.end.is_empty() {
            &mut self.inner_width
        } else {
            &mut self.end_width
        };
        *wmut += width;
    }
}
impl EntryPos for GraphRangePath {
    fn get_entry_pos(&self) -> usize {
        self.start.entry().sub_index
    }
}
impl GraphEntry for GraphRangePath {
    fn get_entry_location(&self) -> ChildLocation {
        self.start.entry()
    }
}
impl HasStartPath for GraphRangePath {
    fn get_start_path(&self) -> &[ChildLocation] {
        self.start.path()
    }
}
impl GraphStart for GraphRangePath {}
impl ExitPos for GraphRangePath {
    fn get_exit_pos(&self) -> usize {
        self.exit
    }
}
impl GraphExit for GraphRangePath {
    fn get_exit_location(&self) -> ChildLocation {
        self.start.entry()
            .into_pattern_location()
            .to_child_location(self.exit)
    }
}
impl HasEndPath for GraphRangePath {
    fn get_end_path(&self) -> &[ChildLocation] {
        self.end.borrow()
    }
}
impl GraphEnd for GraphRangePath {}
impl EndPathMut for GraphRangePath {
    fn end_path_mut(&mut self) -> &mut ChildPath {
        &mut self.end
    }
}
impl ExitMut for GraphRangePath {
    fn exit_mut(&mut self) -> &mut usize {
        &mut self.exit
    }
}
impl End for GraphRangePath {
    fn get_end<
        'a: 'g,
        'g,
        T: Tokenize + 'a,
        D: MatchDirection + 'a,
        Trav: Traversable<'a, 'g, T>,
    >(&self, trav: &'a Trav) -> Child {
        self.get_graph_end::<_, D, _>(trav)
    }
}

impl PathFinished for GraphRangePath {
    fn is_finished(&self) -> bool {
        false
    }
    fn set_finished(&mut self) {
    }
}
impl AdvanceablePath for GraphRangePath {
    fn prev_pos<
        'a: 'g,
        'g,
        T: Tokenize + 'a,
        D: MatchDirection + 'a,
        Trav: Traversable<'a, 'g, T>,
    >(&self, trav: &'a Trav) -> Option<usize> {
        let location = self.get_end_location();
        let pattern = trav.graph().expect_pattern_at(&location);
        D::pattern_index_prev(pattern, location.sub_index)
    }
    fn next_exit_pos<
        'a: 'g,
        'g,
        T: Tokenize + 'a,
        D: MatchDirection + 'a,
        Trav: Traversable<'a, 'g, T>,
    >(&self, trav: &'a Trav) -> Option<usize> {
        self.next_pos::<_, D, _>(trav)
    }
}
impl Wide for GraphRangePath {
    fn width(&self) -> usize {
        self.start.width() + self.inner_width + self.end_width
    }
}