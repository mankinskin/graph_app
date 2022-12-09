pub mod start;
pub mod end;
pub mod query_range_path;
pub mod search;
pub mod overlap_primer;
pub mod prefix_path;
pub mod traversal;
pub mod advance;
pub mod reduce;
pub mod complete;

pub use start::*;
pub use end::*;
pub use query_range_path::*;
pub use search::*;
pub use overlap_primer::*;
pub use prefix_path::*;
pub use traversal::*;
pub use advance::*;
pub use reduce::*;
pub use complete::*;

use crate::{
    vertex::*,
    *,
};
pub trait RelativeDirection<D: MatchDirection> {
    type Direction: MatchDirection;
}
#[derive(Default)]
pub struct Front;
impl<D: MatchDirection> RelativeDirection<D> for Front {
    type Direction = D;
}
#[derive(Default)]
pub struct Back;
impl<D: MatchDirection> RelativeDirection<D> for Back {
    type Direction = <D as MatchDirection>::Opposite;
}

pub trait PathBorder<D: MatchDirection>: PathRoot + HasSinglePath {
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
impl ExitPos for EndPath {
    fn get_exit_pos(&self) -> usize {
        self.entry.sub_index
    }
}
impl ExitPos for SearchPath {
    fn get_exit_pos(&self) -> usize {
        self.end.get_exit_pos()
    }
}
impl<P: ExitPos> ExitPos for OriginPath<P> {
    fn get_exit_pos(&self) -> usize {
        self.postfix.get_exit_pos()
    }
}
//impl<T: GraphExit> ExitPos for T {
//    fn get_exit_pos(&self) -> usize {
//        self.get_exit_location().sub_index
//    }
//}
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

pub trait GraphEntry: EntryPos + Send + Sync {
    fn entry(&self) -> ChildLocation;
    fn get_entry_pattern<
        'a: 'g,
        'g,
        T: Tokenize,
        Trav: Traversable<T>,
    >(&self, trav: &'a Trav) -> Pattern {
        trav.graph().expect_pattern_at(self.entry())
    }
    fn get_entry<
        'a: 'g,
        'g,
        T: Tokenize,
        D: MatchDirection,
        Trav: Traversable<T>,
    >(&self, trav: &'a Trav) -> Child {
        trav.graph().expect_child_at(self.entry())
    }
}
//impl<T: GraphEntry> GraphRoot for T {
//    fn root(&self) -> Child {
//        self.entry().parent
//    }
//}
impl<T: GraphEntry> EntryPos for T {
    fn get_entry_pos(&self) -> usize {
        self.entry().sub_index
    }
}

pub trait GraphExit: ExitPos + Send + Sync {
    fn get_exit_location(&self) -> ChildLocation;
    fn get_exit_pattern<
        'a: 'g,
        'g,
        T: Tokenize,
        Trav: Traversable<T>,
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
        Trav: Traversable<T>,
    >(&self, trav: &'a Trav) -> Option<Child> {
        trav.graph().get_child_at(self.get_exit_location()).ok()
    }
}
impl<P: GraphExit> GraphExit for OriginPath<P> {
    fn get_exit_location(&self) -> ChildLocation {
        self.postfix.get_exit_location()
    }
}
pub trait HasStartPath {
    fn start_path(&self) -> &[ChildLocation];
    fn num_path_segments(&self) -> usize {
        1 + self.start_path().len()
    }
}
impl<P: HasStartPath> HasStartPath for OriginPath<P> {
    fn start_path(&self) -> &[ChildLocation] {
        self.postfix.start_path()
    }
}

pub trait HasEndPath {
    fn end_path(&self) -> &[ChildLocation];
    fn num_path_segments(&self) -> usize {
        1 + self.end_path().len()
    }
}
impl<P: HasEndPath> HasEndPath for OriginPath<P> {
    fn end_path(&self) -> &[ChildLocation] {
        self.postfix.end_path()
    }
}
pub trait HasStartMatchPath: GraphEntry {
    fn start_match_path(&self) -> &StartPath;
    fn start_match_path_mut(&mut self) -> &mut StartPath;
}
impl HasStartMatchPath for StartPath {
    fn start_match_path(&self) -> &StartPath {
        self
    }
    fn start_match_path_mut(&mut self) -> &mut StartPath {
        self
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
impl<P: HasStartMatchPath> HasStartMatchPath for OriginPath<P> {
    fn start_match_path(&self) -> &StartPath {
        self.postfix.start_match_path()
    }
    fn start_match_path_mut(&mut self) -> &mut StartPath {
        self.postfix.start_match_path_mut()
    }
}
pub trait HasEndMatchPath: GraphEntry {
    fn end_match_path(&self) -> &EndPath;
    fn end_match_path_mut(&mut self) -> &mut EndPath;
}
impl HasEndMatchPath for EndPath {
    fn end_match_path(&self) -> &EndPath {
        self
    }
    fn end_match_path_mut(&mut self) -> &mut EndPath {
        self
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
impl<P: HasEndMatchPath> HasEndMatchPath for OriginPath<P> {
    fn end_match_path(&self) -> &EndPath {
        self.postfix.end_match_path()
    }
    fn end_match_path_mut(&mut self) -> &mut EndPath {
        self.postfix.end_match_path_mut()
    }
}
pub trait HasMatchPaths: HasStartMatchPath + HasEndMatchPath {
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


pub trait PatternStart: PatternEntry + HasStartPath  + Send + Sync {
    fn get_start<
        'a: 'g,
        'g,
        T: Tokenize,
        Trav: Traversable<T>,
    >(&self, trav: &'a Trav) -> Child {
        if let Some(next) = self.start_path().last() {
            trav.graph().expect_child_at(next)
        } else {
            self.get_entry()
        }
    }
}

pub trait PatternEnd: PatternExit + HasEndPath + End + Send + Sync {
    fn get_pattern_end<
        'a: 'g,
        'g,
        T: Tokenize,
        Trav: Traversable<T>,
    >(&self, trav: &'a Trav) -> Option<Child> {
        if let Some(end) = self.end_path().last() {
            trav.graph().get_child_at(end).ok()
        } else {
            self.get_exit()
        }
    }
}

pub trait GraphStart: GraphEntry + HasStartPath {
    fn get_start_location(&self) -> ChildLocation {
        if let Some(start) = self.start_path().last() {
            *start
        } else {
            self.entry()
        }
    }
    fn get_start<
        'a: 'g,
        'g,
        T: Tokenize,
        Trav: Traversable<T>,
    >(&self, trav: &'a Trav) -> Option<Child> {
        trav.graph().get_child_at(self.get_start_location()).ok()
    }
}

pub trait GraphEnd: GraphExit + HasEndPath + End {
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
        Trav: Traversable<T>,
    >(&self, trav: &'a Trav) -> Option<Child> {
        trav.graph().get_child_at(self.get_end_location()).ok()
    }
}
impl<T: GraphExit + HasEndPath> GraphEnd for T {}

pub trait EndPathMut {
    fn end_path_mut(&mut self) -> &mut ChildPath;
    fn push_end(&mut self, next: ChildLocation) {
        self.end_path_mut().push(next)
    }
}
impl EndPathMut for OverlapPrimer {
    fn end_path_mut(&mut self) -> &mut ChildPath {
        if self.exit == 0 {
            &mut self.end
        } else {
            &mut self.context.end
        }
    }
}
impl EndPathMut for PrefixQuery {
    fn end_path_mut(&mut self) -> &mut ChildPath {
        &mut self.end
    }
}
impl EndPathMut for QueryRangePath {
    fn end_path_mut(&mut self) -> &mut ChildPath {
        &mut self.end
    }
}
pub trait ExitMut: ExitPos {
    fn exit_mut(&mut self) -> &mut usize;
}
impl ExitMut for EndPath {
    fn exit_mut(&mut self) -> &mut usize {
        &mut self.entry.sub_index
    }
}
impl ExitMut for OverlapPrimer {
    fn exit_mut(&mut self) -> &mut usize {
        &mut self.exit
    }
}
impl ExitMut for QueryRangePath {
    fn exit_mut(&mut self) -> &mut usize {
        &mut self.exit
    }
}
impl ExitMut for PrefixQuery {
    fn exit_mut(&mut self) -> &mut usize {
        &mut self.exit
    }
}
impl ExitMut for SearchPath {
    fn exit_mut(&mut self) -> &mut usize {
        self.end.exit_mut()
    }
}
impl<P: ExitMut> ExitMut for OriginPath<P> {
    fn exit_mut(&mut self) -> &mut usize {
        self.postfix.exit_mut()
    }
}

pub trait End {
    fn get_end<
        'a: 'g,
        'g,
        T: Tokenize,
        D: MatchDirection,
        Trav: Traversable<T>,
    >(&self, trav: &'a Trav) -> Option<Child>;
}

impl End for QueryRangePath {
    fn get_end<
        'a: 'g,
        'g,
        T: Tokenize,
        D: MatchDirection,
        Trav: Traversable<T>,
    >(&self, trav: &'a Trav) -> Option<Child> {
        self.get_pattern_end(trav)
    }
}
impl EndPathMut for SearchPath {
    fn end_path_mut(&mut self) -> &mut ChildPath {
        &mut self.end.path
    }
}
impl<P: EndPathMut> EndPathMut for OriginPath<P> {
    fn end_path_mut(&mut self) -> &mut ChildPath {
        self.postfix.end_path_mut()
    }
}

impl<A: GraphEnd> End for A {
    fn get_end<
        'a: 'g,
        'g,
        T: Tokenize,
        D: MatchDirection,
        Trav: Traversable<T>,
    >(&self, trav: &'a Trav) -> Option<Child> {
        self.get_graph_end(trav)
    }
}


impl HasSinglePath for EndPath {
    fn single_path(&self) -> &[ChildLocation] {
        self.end_path()
    }
}
impl PathRoot for EndPath {
    fn root(&self) -> ChildLocation {
        self.get_exit_location()
    }
}
impl GraphExit for EndPath {
    fn get_exit_location(&self) -> ChildLocation {
        self.entry
    }
}
impl HasEndPath for EndPath {
    fn end_path(&self) -> &[ChildLocation] {
        self.path.borrow()
    }
}
impl EndPathMut for EndPath {
    fn end_path_mut(&mut self) -> &mut ChildPath {
        self.path.borrow_mut()
    }
}
impl WideMut for EndPath {
    fn width_mut(&mut self) -> &mut usize {
        &mut self.width
    }
}
impl Wide for EndPath {
    fn width(&self) -> usize {
        self.width
    }
}