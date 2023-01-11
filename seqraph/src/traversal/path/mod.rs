pub mod structs;
pub mod accessors;
pub mod mutators;

pub use structs::*;
pub use accessors::*;
pub use mutators::*;

use crate::{
    vertex::*,
    *,
};
pub type LocationPath = Vec<ChildLocation>;

use std::hash::Hash;

pub trait BaseQuery:
    Advance
    + Debug
    + Clone
    + Hash
    + PartialEq
    + Eq
    + Send
    + Sync
    + 'static
{}
impl<T:
    Advance
    + Debug
    + Clone
    + Hash
    + PartialEq
    + Eq
    + Send
    + Sync
    + 'static
> BaseQuery for T {}

pub trait BasePath:
    Debug
    + Sized
    + Clone
    + PartialEq
    + Eq
    + Send
    + Sync
    + Unpin
    + 'static
{}
impl<T:
    Debug
    + Sized
    + Clone
    + PartialEq
    + Eq
    + Send
    + Sync
    + Unpin
    + 'static
> BasePath for T {}


//impl<R> GraphRootChild<R> for PrefixQuery {
//    fn child_location(&self) -> ChildLocation {
//    }
//}
//pub trait PatternRootChild<End>: RootChildPos<End> {
//    fn get_pattern(&self) -> &[Child];
//    fn get_exit(&self) -> Option<Child> {
//        self.get_pattern()
//            .get(self.root_child_pos())
//            .cloned()
//    }
//}
//impl GraphStart for SearchPath {}
//impl GraphExit for SearchPath {
//    fn child_location(&self) -> ChildLocation {
//        self.end.child_location()
//    }
//}

//pub trait GraphExit: Send + Sync {
//    fn child_location(&self) -> ChildLocation;
//    fn get_pattern<
//        'a: 'g,
//        'g,
//        T: Tokenize,
//        Trav: Traversable<T>,
//    >(&self, trav: &'a Trav) -> Pattern {
//        trav.graph().expect_pattern_at(self.child_location())
//    }
//    fn root(&self) -> Child {
//        self.child_location().parent
//    }
//    fn get_exit<
//        'a: 'g,
//        'g,
//        T: Tokenize,
//        D: MatchDirection,
//        Trav: Traversable<T>,
//    >(&self, trav: &'a Trav) -> Option<Child> {
//        trav.graph().get_child_at(self.child_location()).ok()
//    }
//}
//impl HasRolePath<Start> for PrefixQuery {
//    fn role_path(&self) -> &[ChildLocation] {
//        &[]
//    }
//}

//impl<T: HasRolePath<End>> T {
//    fn role_path(&self) -> &ChildPath {
//        self.role_path()
//    }
//}
//impl<T: HasRolePath<Start>> T {
//    fn role_path(&self) -> &ChildPath {
//        self.role_path()
//    }
//}

//pub trait HasEndMatchPath: GraphRootChild {
//    fn role_path(&self) -> &ChildPath;
//    fn role_path_mut(&mut self) -> &mut ChildPath;
//}
//impl HasEndMatchPath for ChildPath {
//    fn role_path(&self) -> &ChildPath {
//        self
//    }
//    fn role_path_mut(&mut self) -> &mut ChildPath {
//        self
//    }
//}
//impl HasEndMatchPath for SearchPath {
//    fn role_path(&self) -> &ChildPath {
//        &self.end
//    }
//    fn role_path_mut(&mut self) -> &mut ChildPath {
//        &mut self.end
//    }
//}
//impl<P: HasEndMatchPath> HasEndMatchPath for OriginPath<P> {
//    fn role_path(&self) -> &ChildPath {
//        self.postfix.role_path()
//    }
//    fn role_path_mut(&mut self) -> &mut ChildPath {
//        self.postfix.role_path_mut()
//    }
//}
//pub trait PatternEnd: PatternRootChild<End> + HasRolePath + End + Send + Sync {
//    fn get_pattern_end<
//        'a: 'g,
//        'g,
//        T: Tokenize,
//        Trav: Traversable<T>,
//    >(&self, trav: &'a Trav) -> Option<Child> {
//        if let Some(end) = self.role_path().last() {
//            trav.graph().get_child_at(end).ok()
//        } else {
//            self.get_exit()
//        }
//    }
//}

//pub trait GraphEnd: GraphExit + HasRolePath<End> + End {
//    fn path_child_location(&self) -> ChildLocation {
//        if let Some(end) = self.role_path().role_path().last() {
//            *end
//        } else {
//            self.child_location()
//        }
//    }
//    fn get_graph_end<
//        'a: 'g,
//        'g,
//        T: Tokenize,
//        Trav: Traversable<T>,
//    >(&self, trav: &'a Trav) -> Option<Child> {
//        trav.graph().get_child_at(self.path_child_location()).ok()
//    }
//}
//impl<T: GraphExit + HasRolePath<End>> GraphEnd for T {}

//pub trait HasRolePath<End> {
//    fn role_path_mut(&mut self) -> &mut LocationPath;
//    fn path_mut().push(&mut self, next: ChildLocation) {
//        self.role_path_mut().push(next)
//    }
//}
//impl HasRolePath<End> for OverlapPrimer {
//    fn role_path_mut(&mut self) -> &mut LocationPath {
//        if self.exit == 0 {
//            &mut self.end
//        } else {
//            &mut self.context.end
//        }
//    }
//}
//impl HasRolePath<End> for PrefixQuery {
//    fn role_path_mut(&mut self) -> &mut LocationPath {
//        &mut self.end
//    }
//}
//impl HasRolePath<End> for QueryRangePath {
//    fn role_path_mut(&mut self) -> &mut LocationPath {
//        &mut self.end
//    }
//}
//
//pub trait End {
//    fn path_child<
//        'a: 'g,
//        'g,
//        T: Tokenize,
//        D: MatchDirection,
//        Trav: Traversable<T>,
//    >(&self, trav: &'a Trav) -> Option<Child>;
//}
//
//impl End for QueryRangePath {
//    fn path_child<
//        'a: 'g,
//        'g,
//        T: Tokenize,
//        D: MatchDirection,
//        Trav: Traversable<T>,
//    >(&self, trav: &'a Trav) -> Option<Child> {
//        self.get_pattern_end(trav)
//    }
//}
//impl HasRolePath<End> for SearchPath {
//    fn role_path_mut(&mut self) -> &mut LocationPath {
//        &mut self.end.path
//    }
//}
//impl<P: HasRolePath<End>> HasRolePath<End> for OriginPath<P> {
//    fn role_path_mut(&mut self) -> &mut LocationPath {
//        self.postfix.role_path_mut()
//    }
//}
//
//impl<A: GraphEnd> End for A {
//    fn path_child<
//        'a: 'g,
//        'g,
//        T: Tokenize,
//        D: MatchDirection,
//        Trav: Traversable<T>,
//    >(&self, trav: &'a Trav) -> Option<Child> {
//        self.get_graph_end(trav)
//    }
//}
//impl GraphExit for LocationPath {
//    fn child_location(&self) -> ChildLocation {
//        self.entry
//    }
//}
//impl HasRolePath<End> for LocationPath {
//    fn role_path_mut(&mut self) -> &mut LocationPath {
//        self.path.borrow_mut()
//    }
//}

//impl GraphRootPattern for ChildPath {
//    fn root(&self) -> ChildLocation {
//        self.child_location()
//    }
//}
//impl HasRolePath for ChildPath {
//    fn role_path(&self) -> &[ChildLocation] {
//        self.path.borrow()
//    }
//}
//impl WideMut for ChildPath {
//    fn width_mut(&mut self) -> &mut usize {
//        &mut self.width
//    }
//}
//impl Wide for ChildPath {
//    fn width(&self) -> usize {
//        self.width
//    }
//}