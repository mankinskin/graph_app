pub(crate) mod start;
pub(crate) mod end;
pub(crate) mod query_range_path;
pub(crate) mod search;
pub(crate) mod overlap_primer;
pub(crate) mod prefix_path;
pub(crate) mod traversal;
pub(crate) mod advanceable;
pub(crate) mod reducible;
pub(crate) mod indexing;

pub(crate) use start::*;
pub(crate) use end::*;
pub(crate) use query_range_path::*;
pub(crate) use search::*;
pub(crate) use overlap_primer::*;
pub(crate) use prefix_path::*;
pub(crate) use traversal::*;
pub(crate) use advanceable::*;
pub(crate) use reducible::*;
pub(crate) use indexing::*;

use crate::{
    vertex::*,
    *,
};
pub trait RelativeDirection<D: MatchDirection> {
    type Direction: MatchDirection;
}
#[derive(Default)]
pub(crate) struct Front;
impl<D: MatchDirection> RelativeDirection<D> for Front {
    type Direction = D;
}
#[derive(Default)]
pub(crate) struct Back;
impl<D: MatchDirection> RelativeDirection<D> for Back {
    type Direction = <D as MatchDirection>::Opposite;
}

pub(crate) trait BorderPath: Wide {
    fn path(&self) -> &[ChildLocation];
    fn is_perfect(&self) -> bool {
        self.path().is_empty()
    }
    fn entry(&self) -> ChildLocation;
    fn pattern<
        'a: 'g,
        'g,
        T: Tokenize,
        Trav: Traversable<'a, 'g, T>,
    >(&self, trav: &'a Trav) -> Pattern {
        trav.graph().expect_pattern_at(self.entry())
    }
}
pub(crate) trait DirectedBorderPath<D: MatchDirection>: BorderPath {
    type BorderDirection: RelativeDirection<D>;
    fn pattern_entry_outer_pos<P: IntoPattern>(pattern: P, entry: usize) -> Option<usize> {
        <Self::BorderDirection as RelativeDirection<D>>::Direction::pattern_index_next(pattern, entry)
    }
    fn pattern_outer_pos<P: IntoPattern>(&self, pattern: P) -> Option<usize> {
        Self::pattern_entry_outer_pos(pattern, self.entry().sub_index)
    }
    fn is_at_pattern_border<P: IntoPattern>(&self, pattern: P) -> bool {
        self.pattern_outer_pos(pattern).is_none()
    }
    fn pattern_is_complete<P: IntoPattern>(&self, pattern: P) -> bool {
        self.is_perfect() && self.is_at_pattern_border(pattern)
    }
}
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
pub trait GraphEntry {
    fn get_entry_location(&self) -> ChildLocation;
    fn get_entry_pattern<
        'a: 'g,
        'g,
        T: Tokenize,
        Trav: Traversable<'a, 'g, T>,
    >(&self, trav: &'a Trav) -> Pattern {
        trav.graph().expect_pattern_at(self.get_entry_location())
    }
    fn get_entry<
        'a: 'g,
        'g,
        T: Tokenize,
        D: MatchDirection,
        Trav: Traversable<'a, 'g, T>,
    >(&self, trav: &'a Trav) -> Child {
        trav.graph().expect_child_at(self.get_entry_location())
    }
}
impl<T: GraphEntry> EntryPos for T {
    fn get_entry_pos(&self) -> usize {
        self.get_entry_location().sub_index
    }
}
pub trait GraphExit {
    fn get_exit_location(&self) -> ChildLocation;
    fn get_exit_pattern<
        'a: 'g,
        'g,
        T: Tokenize,
        Trav: Traversable<'a, 'g, T>,
    >(&self, trav: &'a Trav) -> Pattern {
        trav.graph().expect_pattern_at(self.get_exit_location())
    }
    fn get_exit<
        'a: 'g,
        'g,
        T: Tokenize,
        D: MatchDirection,
        Trav: Traversable<'a, 'g, T>,
    >(&self, trav: &'a Trav) -> Child {
        trav.graph().expect_child_at(self.get_exit_location())
    }
}
impl<T: GraphExit> ExitPos for T {
    fn get_exit_pos(&self) -> usize {
        self.get_exit_location().sub_index
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
pub trait PathComplete: GraphEntry + HasStartPath + HasEndPath + ExitPos {
    fn is_complete<
        'a: 'g,
        'g,
        T: Tokenize,
        D: MatchDirection,
        Trav: Traversable<'a, 'g, T>,
    >(&self, trav: &'a Trav) -> bool {
        let pattern = self.get_entry_pattern(trav);
        DirectedBorderPath::pattern_is_complete(&self, &pattern[..]) &&
            self.get_end_path().is_empty() &&
            <EndPath as DirectedBorderPath<D>>::pattern_entry_outer_pos(pattern, self.get_exit_pos()).is_none()
    }
}
pub trait PatternStart: PatternEntry + HasStartPath {
    fn get_start<
        'a: 'g,
        'g,
        T: Tokenize,
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
        T: Tokenize,
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
            *start
        } else {
            self.get_entry_location()
        }
    }
    fn get_start<
        'a: 'g,
        'g,
        T: Tokenize,
        Trav: Traversable<'a, 'g, T>,
    >(&self, trav: &'a Trav) -> Child {
        trav.graph().expect_child_at(self.get_start_location())
    }
}
pub trait GraphEnd: GraphExit + HasEndPath + End {
    fn get_end_location(&self) -> ChildLocation {
        if let Some(end) = self.get_end_path().last() {
            *end
        } else {
            self.get_exit_location()
        }
    }
    fn get_graph_end<
        'a: 'g,
        'g,
        T: Tokenize,
        Trav: Traversable<'a, 'g, T>,
    >(&self, trav: &'a Trav) -> Child {
        trav.graph().expect_child_at(self.get_end_location())
    }
}
pub trait EndPathMut {
    fn end_path_mut(&mut self) -> &mut ChildPath;
    fn push_end(&mut self, next: ChildLocation) {
        self.end_path_mut().push(next)
    }
}
pub trait ExitMut: ExitPos {
    fn exit_mut(&mut self) -> &mut usize;
}
pub trait AdvanceableExit: ExitPos + ExitMut + PathFinished {
    fn next_exit_pos<
        'a: 'g,
        'g,
        T: Tokenize,
        D: MatchDirection,
        Trav: Traversable<'a, 'g, T>,
    >(&self, _trav: &'a Trav) -> Option<usize>;
    fn advance_exit_pos<
        'a: 'g,
        'g,
        T: Tokenize,
        D: MatchDirection,
        Trav: Traversable<'a, 'g, T>,
    >(&mut self, trav: &'a Trav) -> Result<(), ()> {
        if let Some(next) = self.next_exit_pos::<_, D, _>(trav) {
            *self.exit_mut() = next;
            Ok(())
        } else {
            self.set_finished();
            Err(())
        }
    }
}
impl<P: ExitMut + PatternExit + PathFinished> AdvanceableExit for P {
    fn next_exit_pos<
        'a: 'g,
        'g,
        T: Tokenize,
        D: MatchDirection,
        Trav: Traversable<'a, 'g, T>,
    >(&self, _trav: &'a Trav) -> Option<usize> {
        D::pattern_index_next(self.get_exit_pattern(), self.get_exit_pos())
    }
}
pub trait End {
    fn get_end<
        'a: 'g,
        'g,
        T: Tokenize,
        D: MatchDirection,
        Trav: Traversable<'a, 'g, T>,
    >(&self, trav: &'a Trav) -> Child;
}