use crate::*;

pub mod pos;
pub use pos::*;

pub mod root;
pub use root::*;

pub trait LeafChild<R>: RootChildPos<R> {
    fn leaf_child_location(&self) -> Option<ChildLocation>;
    fn leaf_child<
        T: Tokenize,
        Trav: Traversable<T>,
    >(&self, trav: &Trav) -> Child;
}
impl<R: PathRole, P: RootChild<R> + PathChild<R>> LeafChild<R> for P {
    fn leaf_child_location(&self) -> Option<ChildLocation> {
        self.path_child_location()
    }
    fn leaf_child<
        T: Tokenize,
        Trav: Traversable<T>,
    >(&self, trav: &Trav) -> Child {
        self.path_child(trav)
            .unwrap_or_else(||
                self.root_child(trav)
            )
    }
}

/// used to get a descendant in a Graph, pointed to by a child path
trait PathChild<R: PathRole>: HasPath<R> {
    fn path_child_location(&self) -> Option<ChildLocation> {
        R::bottom_up_iter(self.path().iter()).next().cloned()
    }
    fn path_child<
        T: Tokenize,
        Trav: Traversable<T>,
    >(&self, trav: &Trav) -> Option<Child> {
        self.path_child_location().map(|loc|
            trav.graph().expect_child_at(loc)
        )
    }
}
impl<R: PathRole> PathChild<R> for QueryRangePath
    where QueryRangePath: HasPath<R> + PatternRootChild<R>
{
}
impl<R: PathRole> PathChild<R> for RolePath<R> {
}
impl<R: PathRole> PathChild<R> for SearchPath
    where SearchPath: HasRolePath<R>
{
    fn path_child<
        T: Tokenize,
        Trav: Traversable<T>,
    >(&self, trav: &Trav) -> Option<Child> {
        PathChild::<R>::path_child(self.role_path(), trav)
    }
}
//impl PathChild<End> for OverlapPrimer {
//    fn path_child<
//        'a: 'g,
//        'g,
//        T: Tokenize,
//        Trav: Traversable<T>,
//    >(&self, trav: &'a Trav) -> Option<Child> {
//        self.context.path_child(trav)
//    }
//}
//impl PathChild<End> for PrefixQuery {
//    fn path_child<
//        'a: 'g,
//        'g,
//        T: Tokenize,
//        Trav: Traversable<T>,
//    >(&self, trav: &'a Trav) -> Option<Child> {
//        if let Some(end) = &self.end {
//            PathChild::<End>::path_child(end, trav)
//        } else {
//            Some(RootChild::<End>::root_child(self, trav))
//        }
//    }
//}
//impl<R: PathRole, P: PathChild<R>> PathChild<R> for OriginPath<P> {
//    fn path_child<
//        T: Tokenize,
//        Trav: Traversable<T>,
//    >(&self, trav: &Trav) -> Option<Child> {
//        self.postfix.path_child(trav)
//    }
//}