pub(crate) mod start;
pub(crate) mod end;
pub(crate) mod query_range_path;
pub(crate) mod search;
pub(crate) mod overlap_primer;
pub(crate) mod prefix_path;
pub(crate) mod traversal;
pub(crate) mod advanceable;
pub(crate) mod reduce;
pub(crate) mod complete;
//pub(crate) mod indexing;

pub(crate) use start::*;
pub(crate) use end::*;
pub(crate) use query_range_path::*;
pub(crate) use search::*;
pub(crate) use overlap_primer::*;
pub(crate) use prefix_path::*;
pub(crate) use traversal::*;
pub(crate) use advanceable::*;
pub(crate) use reduce::*;
pub(crate) use complete::*;
//pub(crate) use indexing::*;

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

pub(crate) trait PathBorder<D: MatchDirection>: PathRoot + HasSinglePath {
    type BorderDirection: RelativeDirection<D>;

    fn pattern_entry_outer_pos<P: IntoPattern>(pattern: P, entry: usize) -> Option<usize> {
        <Self::BorderDirection as RelativeDirection<D>>::Direction::pattern_index_next(pattern, entry)
    }
    fn pattern_outer_pos<P: IntoPattern>(&self, pattern: P) -> Option<usize> {
        Self::pattern_entry_outer_pos(pattern, self.root().sub_index)
    }
    fn is_at_pattern_border<P: IntoPattern>(&self, pattern: P) -> bool {
        self.pattern_outer_pos(pattern).is_none()
    }
    fn pattern_is_complete<P: IntoPattern>(&self, pattern: P) -> bool {
        self.single_path().is_empty() && self.is_at_pattern_border(pattern)
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
    fn get_exit(&self) -> Option<Child> {
        self.get_exit_pattern()
            .get(self.get_exit_pos())
            .cloned()
    }
}
pub trait RootChild {
    fn root_child(&self) -> Child;
}
pub trait PathRoot {
    fn root(&self) -> ChildLocation;
}
pub trait HasSinglePath {
    fn single_path(&self) -> &[ChildLocation];
}
impl<T: PathRoot> RootChild for T {
    fn root_child(&self) -> Child {
        self.root().parent
    }
}
pub trait GraphEntry: EntryPos {
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
//impl<T: GraphEntry> GraphRoot for T {
//    fn root(&self) -> Child {
//        self.get_entry_location().parent
//    }
//}
impl<T: GraphEntry> EntryPos for T {
    fn get_entry_pos(&self) -> usize {
        self.get_entry_location().sub_index
    }
}
pub trait GraphExit: ExitPos {
    fn get_exit_location(&self) -> ChildLocation;
    fn get_exit_pattern<
        'a: 'g,
        'g,
        T: Tokenize,
        Trav: Traversable<'a, 'g, T>,
    >(&self, trav: &'a Trav) -> Pattern {
        trav.graph().expect_pattern_at(self.get_exit_location())
    }
    fn root(&self) -> Child {
        self.get_exit_location().parent
    }
    fn get_exit<
        'a: 'g,
        'g,
        T: Tokenize,
        D: MatchDirection,
        Trav: Traversable<'a, 'g, T>,
    >(&self, trav: &'a Trav) -> Option<Child> {
        trav.graph().get_child_at(self.get_exit_location()).ok()
    }
}
impl<T: GraphExit> ExitPos for T {
    fn get_exit_pos(&self) -> usize {
        self.get_exit_location().sub_index
    }
}
pub(crate) trait HasStartPath {
    fn start_path(&self) -> &[ChildLocation];
    fn num_path_segments(&self) -> usize {
        1 + self.start_path().len()
    }
}
pub(crate) trait HasEndPath {
    fn end_path(&self) -> &[ChildLocation];
    fn num_path_segments(&self) -> usize {
        1 + self.end_path().len()
    }
}
pub(crate) trait HasStartMatchPath {
    fn start_match_path(&self) -> &StartPath;
    fn start_match_path_mut(&mut self) -> &mut StartPath;
}
pub(crate) trait HasEndMatchPath {
    fn end_match_path(&self) -> &EndPath;
    fn end_match_path_mut(&mut self) -> &mut EndPath;
}
pub(crate) trait HasMatchPaths: HasStartMatchPath + HasEndMatchPath {
    fn into_paths(self) -> (StartPath, EndPath);
    fn num_path_segments(&self) -> usize {
        self.start_match_path().num_path_segments() + self.end_match_path().num_path_segments()
    }
    fn min_path_segments(&self) -> usize {
        self.start_match_path().num_path_segments().min(self.end_match_path().num_path_segments())
    }
    //fn root(&self) -> Child {
    //    self.start_match_path().root()
    //}
}

pub(crate) trait PatternStart: PatternEntry + HasStartPath {
    fn get_start<
        'a: 'g,
        'g,
        T: Tokenize,
        Trav: Traversable<'a, 'g, T>,
    >(&self, trav: &'a Trav) -> Child {
        if let Some(next) = self.start_path().last() {
            trav.graph().expect_child_at(next)
        } else {
            self.get_entry()
        }
    }
}
pub(crate) trait PatternEnd: PatternExit + HasEndPath + End {
    fn get_pattern_end<
        'a: 'g,
        'g,
        T: Tokenize,
        Trav: Traversable<'a, 'g, T>,
    >(&self, trav: &'a Trav) -> Option<Child> {
        if let Some(end) = self.end_path().last() {
            trav.graph().get_child_at(end).ok()
        } else {
            self.get_exit()
        }
    }
}
pub(crate) trait GraphStart: GraphEntry + HasStartPath {
    fn get_start_location(&self) -> ChildLocation {
        if let Some(start) = self.start_path().last() {
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
    >(&self, trav: &'a Trav) -> Option<Child> {
        trav.graph().get_child_at(self.get_start_location()).ok()
    }
}
pub(crate) trait GraphEnd: GraphExit + HasEndPath + End {
    fn get_end_location(&self) -> ChildLocation {
        if let Some(end) = self.end_path().last() {
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
    >(&self, trav: &'a Trav) -> Option<Child> {
        trav.graph().get_child_at(self.get_end_location()).ok()
    }
}
pub(crate) trait EndPathMut {
    fn end_path_mut(&mut self) -> &mut ChildPath;
    fn push_end(&mut self, next: ChildLocation) {
        self.end_path_mut().push(next)
    }
}
pub(crate) trait ExitMut: ExitPos {
    fn exit_mut(&mut self) -> &mut usize;
}
pub(crate) trait End {
    fn get_end<
        'a: 'g,
        'g,
        T: Tokenize,
        D: MatchDirection,
        Trav: Traversable<'a, 'g, T>,
    >(&self, trav: &'a Trav) -> Option<Child>;
}