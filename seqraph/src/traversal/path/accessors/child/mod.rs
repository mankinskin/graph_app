use crate::*;

pub mod pos;
pub use pos::*;

pub trait LeafChild<R>: ChildPos<R> {
    fn leaf_child<
        'a: 'g,
        'g,
        T: Tokenize,
        Trav: Traversable<T>,
    >(&self, trav: &'a Trav) -> Child;
}
impl<R, P: RootChild<R> + PathChild<R>> LeafChild<R> for P {
    fn leaf_child<
        'a: 'g,
        'g,
        T: Tokenize,
        Trav: Traversable<T>,
    >(&self, trav: &'a Trav) -> Child {
        if let Some(c) = self.path_child(trav) {
            c
        } else {
            self.root_child(trav)
        }
    }
}
pub trait RootChild<R>: ChildPos<R> {
    fn root_child<
        'a: 'g,
        'g,
        T: Tokenize,
        Trav: Traversable<T>,
    >(&self, trav: &'a Trav) -> Child;
}
macro_rules! impl_child {
    {
        RootChild for $target:ty, $self_:ident, $trav:ident => $func:expr
    } => {
        impl<R> RootChild<R> for $target
            where $target: ChildPos<R>
        {
            fn root_child<
                'a: 'g,
                'g,
                T: Tokenize,
                Trav: Traversable<T>
            >(& $self_, $trav: &Trav) -> Child {
                $func
            }
        }
    };
}
impl_child! { RootChild for QueryRangePath, self, trav => self.root_child(trav) }
impl_child! { RootChild for PrefixQuery, self, trav => self.root_child(trav) }
impl RootChild<End> for OverlapPrimer {
    fn root_child<
        'a: 'g,
        'g,
        T: Tokenize,
        Trav: Traversable<T>
    >(&self, trav: &Trav) -> Child {
        self.start
    }
}
impl RootChild<Start> for SearchPath {
    fn root_child<
        'a: 'g,
        'g,
        T: Tokenize,
        Trav: Traversable<T>
    >(&self, trav: &Trav) -> Child {
        RootChild::<Start>::root_child(&self.start, trav)
    }
}
impl RootChild<End> for SearchPath {
    fn root_child<
        'a: 'g,
        'g,
        T: Tokenize,
        Trav: Traversable<T>
    >(&self, trav: &Trav) -> Child {
        RootChild::<End>::root_child(&self.end, trav)
    }
}
//impl_child! { RootChild for PathLeaf, self, trav => self.root_child(trav) }
impl_child! { RootChild for ChildPath<R>, self, trav => self.root_child(trav) }

impl<R, P: RootChild<R>> RootChild<R> for OriginPath<P> {
    fn root_child<
        'a: 'g,
        'g,
        T: Tokenize,
        Trav: Traversable<T>,
    >(&self, trav: &Trav) -> Child {
        self.postfix.root_child(trav)
    }
}
impl<P: MatchEndPath> RootChild<Start> for MatchEnd<P> {
    fn root_child<
        'a: 'g,
        'g,
        T: Tokenize,
        Trav: Traversable<T>,
    >(&self, trav: &Trav) -> Child {
        match self {
            Self::Complete(c) => *c,
            Self::Path(path) => path.root_child(trav),
        }
    }
}

/// used to get a direct child in a Graph
pub trait GraphRootChild<R>: GraphRoot + ChildPos<R> {
    fn graph_root_child_location(&self) -> ChildLocation;
    fn graph_root_child_location_mut(&mut self) -> &mut ChildLocation;
    fn graph_root_child<
        'a: 'g,
        'g,
        T: Tokenize,
        Trav: Traversable<T>,
    >(&self, trav: &'a Trav) -> Child {
        trav.graph().expect_child_at(<_ as GraphRootChild<R>>::graph_root_child_location(self))
    }
}

impl<R, P: GraphRootChild<R>> GraphRootChild<R> for OriginPath<P> {
    fn graph_root_child_location(&self) -> ChildLocation {
        <_ as GraphRootChild::<R>>::graph_root_child_location(&self.postfix)
    }
    fn graph_root_child_location_mut(&mut self) -> &mut ChildLocation {
        <_ as GraphRootChild::<R>>::graph_root_child_location_mut(&mut self.postfix)
    }
}
impl GraphRootChild<Start> for SearchPath {
    fn graph_root_child_location(&self) -> ChildLocation {
        <_ as GraphRootChild<Start>>::graph_root_child_location(&self.start)
    }
    fn graph_root_child_location_mut(&mut self) -> &mut ChildLocation {
        <_ as GraphRootChild::<Start>>::graph_root_child_location_mut(&mut self.start)
    }
}
impl GraphRootChild<End> for SearchPath {
    fn graph_root_child_location(&self) -> ChildLocation {
        <_ as GraphRootChild<End>>::graph_root_child_location(&self.end)
    }
    fn graph_root_child_location_mut(&mut self) -> &mut ChildLocation {
        <_ as GraphRootChild::<End>>::graph_root_child_location_mut(&mut self.end)
    }
}
impl<R: PathRole> GraphRootChild<R> for ChildPath<R> {
    fn graph_root_child_location(&self) -> ChildLocation {
        *R::top_down_iter(self.path.iter()).next().unwrap()
    }
    fn graph_root_child_location_mut(&mut self) -> &mut ChildLocation {
        R::top_down_iter(self.path.iter_mut()).next().unwrap()
    }
}
//impl<R> GraphRootChild<R> for PathLeaf {
//    fn graph_root_child_location(&self) -> ChildLocation {
//        self.entry
//    }
//}

/// used to get a direct child of a pattern
pub trait PatternRootChild<R>: ChildPos<R> + PatternRoot {
    fn pattern_root_child(&self) -> Child {
        PatternRoot::pattern_root_pattern(self)[self.child_pos()]
    }
}
impl<R> PatternRootChild<R> for QueryRangePath
    where QueryRangePath: ChildPos<R>
{
}
impl<R> PatternRootChild<R> for PrefixQuery
    where PrefixQuery: ChildPos<R>
{
}
impl PatternRootChild<End> for OverlapPrimer {
}

/// used to get a descendant in a Graph, pointed to by a child path
pub trait PathChild<R>: HasPath<R> {
    fn get_path_child_location(&self) -> Option<ChildLocation> {
        // todo: leaf direction
        self.path().last().copied()
    }
    fn path_child<
        'a: 'g,
        'g,
        T: Tokenize,
        Trav: Traversable<T>,
    >(&self, trav: &'a Trav) -> Option<Child>;
    //{
    //    trav.graph().get_child_at(self.get_child_location()).ok()
    //}
}
impl<R> PathChild<R> for QueryRangePath
    where QueryRangePath: HasPath<R> + PatternRootChild<R>
{
    fn path_child<
        'a: 'g,
        'g,
        T: Tokenize,
        Trav: Traversable<T>,
    >(&self, trav: &'a Trav) -> Option<Child> {
        //if let Some(next) = self.path().last() {
        //    trav.graph().expect_child_at(next)
        //} else {
        //    self.get_child()
        //}
        self.path_child(trav)
    }
}
impl<R: PathRole> PathChild<R> for ChildPath<R> {
    fn path_child<
        'a: 'g,
        'g,
        T: Tokenize,
        Trav: Traversable<T>,
    >(&self, trav: &'a Trav) -> Option<Child> {
        // todo: maybe get this from trav
        Some(self.get_child())
    }
}
impl<R: PathRole> PathChild<R> for SearchPath
    where SearchPath: HasRootedPath<R>
{
    fn path_child<
        'a: 'g,
        'g,
        T: Tokenize,
        Trav: Traversable<T>,
    >(&self, trav: &'a Trav) -> Option<Child> {
        PathChild::<R>::path_child(self.child_path(), trav)
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
impl<R, P: PathChild<R>> PathChild<R> for OriginPath<P> {
    fn path_child<
        'a: 'g,
        'g,
        T: Tokenize,
        Trav: Traversable<T>,
    >(&self, trav: &'a Trav) -> Option<Child> {
        self.postfix.path_child(trav)
    }
}