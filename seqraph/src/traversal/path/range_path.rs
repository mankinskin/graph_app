use std::borrow::Borrow;

use crate::{
    vertex::*,
    *,
};
pub trait EntryPos {
    fn get_entry_pos(&self) -> usize;
}
pub trait ExitPos {
    fn get_exit_pos(&self) -> usize;
}
pub trait PatternEntry: EntryPos {
    fn get_entry_pattern(&self) -> &[Child];
    fn get_entry(&self) -> Child {
        self.get_entry_pattern()[self.get_entry_pos()]
    }
}
pub trait PatternExit: ExitPos {
    fn get_exit_pattern(&self) -> &[Child];
    fn get_exit(&self) -> Child {
        self.get_exit_pattern()[self.get_exit_pos()]
    }
}
pub trait GraphEntry: EntryPos {
    fn get_entry_location(&self) -> ChildLocation;
    fn get_entry_pattern<
        'a: 'g,
        'g,
        T: Tokenize + 'a,
        Trav: Traversable<'a, 'g, T>,
    >(&self, trav: &'a Trav) -> Pattern {
        trav.graph().expect_pattern_at(self.get_entry_location())
    }
    fn get_entry<
        'a: 'g,
        'g,
        T: Tokenize + 'a,
        D: MatchDirection + 'a,
        Trav: Traversable<'a, 'g, T>,
    >(&self, trav: &'a Trav) -> Child {
        trav.graph().expect_child_at(self.get_entry_location())
    }
}
pub trait GraphExit: ExitPos {
    fn get_exit_location(&self) -> ChildLocation;
    fn get_exit_pattern<
        'a: 'g,
        'g,
        T: Tokenize + 'a,
        Trav: Traversable<'a, 'g, T>,
    >(&self, trav: &'a Trav) -> Pattern {
        trav.graph().expect_pattern_at(self.get_exit_location())
    }
    fn get_exit<
        'a: 'g,
        'g,
        T: Tokenize + 'a,
        D: MatchDirection + 'a,
        Trav: Traversable<'a, 'g, T>,
    >(&self, trav: &'a Trav) -> Child {
        trav.graph().expect_child_at(self.get_exit_location())
    }
}
pub trait HasStartPath {
    fn get_start_path(&self) -> &[ChildLocation];
}
pub trait HasEndPath {
    fn get_end_path(&self) -> &[ChildLocation];
}
pub trait PathFinished {
    fn is_finished(&self) -> bool;
    fn set_finished(&mut self);
}
pub trait PatternStart: PatternEntry + HasStartPath {
    fn get_start<
        'a: 'g,
        'g,
        T: Tokenize + 'a,
        D: MatchDirection + 'a,
        Trav: Traversable<'a, 'g, T>,
    >(&self, trav: &'a Trav) -> Child {
        if let Some(next) = self.get_start_path().last() {
            trav.graph().expect_child_at(next)
        } else {
            self.get_entry()
        }
    }
}
pub trait PatternEnd: PatternExit + HasEndPath + End {
    fn get_pattern_end<
        'a: 'g,
        'g,
        T: Tokenize + 'a,
        D: MatchDirection + 'a,
        Trav: Traversable<'a, 'g, T>,
    >(&self, trav: &'a Trav) -> Child {
        if let Some(start) = self.get_end_path().last() {
            trav.graph().expect_child_at(start)
        } else {
            self.get_exit()
        }
    }
}
pub trait GraphStart: GraphEntry + HasStartPath {
    fn get_start_location(&self) -> ChildLocation {
        if let Some(start) = self.get_start_path().last() {
            start.clone()
        } else {
            self.get_entry_location()
        }
    }
    fn get_start<
        'a: 'g,
        'g,
        T: Tokenize + 'a,
        D: MatchDirection + 'a,
        Trav: Traversable<'a, 'g, T>,
    >(&self, trav: &'a Trav) -> Child {
        trav.graph().expect_child_at(self.get_start_location())
    }
}
pub trait GraphEnd: GraphExit + HasEndPath + End {
    fn get_end_location(&self) -> ChildLocation {
        if let Some(end) = self.get_end_path().last() {
            end.clone()
        } else {
            self.get_exit_location()
        }
    }
    fn get_graph_end<
        'a: 'g,
        'g,
        T: Tokenize + 'a,
        D: MatchDirection + 'a,
        Trav: Traversable<'a, 'g, T>,
    >(&self, trav: &'a Trav) -> Child {
        trav.graph().expect_child_at(self.get_end_location())
    }
}
pub trait EndPathMut {
    fn end_path_mut(&mut self) -> &mut ChildPath;
}
pub trait ExitMut {
    fn exit_mut(&mut self) -> &mut usize;
}
pub trait End {
    fn get_end<
        'a: 'g,
        'g,
        T: Tokenize + 'a,
        D: MatchDirection + 'a,
        Trav: Traversable<'a, 'g, T>,
    >(&self, trav: &'a Trav) -> Child;
}
pub trait AdvanceablePath: Clone + EndPathMut + ExitMut + End + PathFinished {
    fn next_exit_pos<
        'a: 'g,
        'g,
        T: Tokenize + 'a,
        D: MatchDirection + 'a,
        Trav: Traversable<'a, 'g, T>,
    >(&self, trav: &'a Trav) -> Option<usize>;
    fn prev_pos<
        'a: 'g,
        'g,
        T: Tokenize + 'a,
        D: MatchDirection + 'a,
        Trav: Traversable<'a, 'g, T>,
    >(&self, trav: &'a Trav) -> Option<usize>;
    fn push_next(&mut self, next: ChildLocation) {
        self.end_path_mut().push(next)
    }
    fn reduce_mismatch<
        'a: 'g,
        'g,
        T: Tokenize + 'a,
        D: MatchDirection + 'a,
        Trav: Traversable<'a, 'g, T>,
    >(mut self, trav: &'a Trav) -> Self {
        let graph = trav.graph();
        // remove segments pointing to mismatch at pattern head
        let end = self.end_path_mut();
        while let Some(mut location) = end.pop() {
            let pattern = graph.expect_pattern_at(&location);
            // skip segments at end of pattern
            if let Some(prev) = D::pattern_index_prev(pattern.borrow(), location.sub_index) {
                location.sub_index = prev;
                end.push(location);
                break;
            }
        }
        if end.is_empty() {
            *self.exit_mut() = self.prev_pos::<_, D, _>(trav).unwrap();
        }
        self
    }
    fn try_advance<
        'a: 'g,
        'g,
        T: Tokenize + 'a,
        D: MatchDirection + 'a,
        Trav: Traversable<'a, 'g, T>,
    >(mut self, trav: &'a Trav) -> Result<Self, Self> {
        let graph = trav.graph();
        // skip path segments with no successors
        let end = self.end_path_mut();
        while let Some(mut location) = end.pop() {
            let pattern = graph.expect_pattern_at(&location);
            if let Some(next) = D::pattern_index_next(pattern, location.sub_index) {
                location.sub_index = next;
                end.push(location);
                return Ok(self);
            }
        }
        // end is empty (exit is prev)
        if let Some(next) = self.next_exit_pos::<_, D, _>(trav) {
            *self.exit_mut() = next;
            Ok(self)
        } else {
            self.set_finished();
            Err(self)
        }
    }
    fn into_advanced<
        'a: 'g,
        'g,
        T: Tokenize + 'a,
        D: MatchDirection + 'a,
        Trav: Traversable<'a, 'g, T>,
    >(self, trav: &'a Trav) -> Self {
        self.try_advance::<_, D, _>(trav).unwrap_or_else(|e| e)
    }
    fn get_advance<
        'a: 'g,
        'g,
        T: Tokenize + 'a,
        D: MatchDirection + 'a,
        Trav: Traversable<'a, 'g, T>,
    >(self, trav: &'a Trav) -> (Child, Self) {
        (
            self.get_end::<_, D, _>(trav),
            self.into_advanced::<_, D, _>(trav),
        )
    }
}