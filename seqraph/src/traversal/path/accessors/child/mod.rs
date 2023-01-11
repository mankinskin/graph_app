use crate::*;

pub mod pos;
pub use pos::*;

pub trait LeafChild<R>: RootChildPos<R> {
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
        self.path_child(trav)
    }
}
pub trait RootChild<R>: RootChildPos<R> {
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
        impl<R: PathRole> RootChild<R> for $target
            where $target: RootChildPos<R>
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
impl_child! { RootChild for QueryRangePath, self, _trav => self.query[self.root_child_pos()] }
//impl_child! { RootChild for PrefixQuery, self, trav => self.root_child(trav) }
//impl RootChild<End> for OverlapPrimer {
//    fn root_child<
//        'a: 'g,
//        'g,
//        T: Tokenize,
//        Trav: Traversable<T>
//    >(&self, trav: &Trav) -> Child {
//        self.start
//    }
//}
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
impl_child! { RootChild for ChildPath<R>, self, trav => trav.graph().expect_child_at(R::top_down_iter(self.path.iter()).next().unwrap()) }

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
pub trait GraphRootChild<R>: GraphRootPattern + RootChildPos<R> {
    fn root_child_location(&self) -> ChildLocation;
    fn root_child_location_mut(&mut self) -> &mut ChildLocation;
    fn graph_root_child<
        'a: 'g,
        'g,
        T: Tokenize,
        Trav: Traversable<T>,
    >(&self, trav: &'a Trav) -> Child {
        trav.graph().expect_child_at(<_ as GraphRootChild<R>>::root_child_location(self))
    }
}

impl<R, P: GraphRootChild<R>> GraphRootChild<R> for OriginPath<P> {
    fn root_child_location(&self) -> ChildLocation {
        <_ as GraphRootChild::<R>>::root_child_location(&self.postfix)
    }
    fn root_child_location_mut(&mut self) -> &mut ChildLocation {
        <_ as GraphRootChild::<R>>::root_child_location_mut(&mut self.postfix)
    }
}
impl GraphRootChild<Start> for SearchPath {
    fn root_child_location(&self) -> ChildLocation {
        <_ as GraphRootChild<Start>>::root_child_location(&self.start)
    }
    fn root_child_location_mut(&mut self) -> &mut ChildLocation {
        <_ as GraphRootChild::<Start>>::root_child_location_mut(&mut self.start)
    }
}
impl GraphRootChild<End> for SearchPath {
    fn root_child_location(&self) -> ChildLocation {
        <_ as GraphRootChild<End>>::root_child_location(&self.end)
    }
    fn root_child_location_mut(&mut self) -> &mut ChildLocation {
        <_ as GraphRootChild::<End>>::root_child_location_mut(&mut self.end)
    }
}
impl<R: PathRole> GraphRootChild<R> for ChildPath<R> {
    fn root_child_location(&self) -> ChildLocation {
        *R::top_down_iter(self.path.iter()).next().unwrap()
    }
    fn root_child_location_mut(&mut self) -> &mut ChildLocation {
        R::top_down_iter(self.path.iter_mut()).next().unwrap()
    }
}
//impl<R> GraphRootChild<R> for PathLeaf {
//    fn root_child_location(&self) -> ChildLocation {
//        self.entry
//    }
//}

/// used to get a direct child of a pattern
pub trait PatternRootChild<R>: RootChildPos<R> + PatternRoot {
    fn pattern_root_child(&self) -> Child {
        PatternRoot::pattern_root_pattern(self)[self.root_child_pos()]
    }
}
impl<R> PatternRootChild<R> for QueryRangePath
    where QueryRangePath: RootChildPos<R>
{
}
//impl<R> PatternRootChild<R> for PrefixQuery
//    where PrefixQuery: RootChildPos<R>
//{
//}
//impl PatternRootChild<End> for OverlapPrimer {
//}

/// used to get a descendant in a Graph, pointed to by a child path
pub trait PathChild<R>: HasPath<R> {
    fn path_child_location(&self) -> Option<ChildLocation> {
        // todo: leaf direction
        self.path().last().copied()
    }
    fn path_child<
        'a: 'g,
        'g,
        T: Tokenize,
        Trav: Traversable<T>,
    >(&self, trav: &'a Trav) -> Child;
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
    >(&self, trav: &'a Trav) -> Child {
        if let Some(next) = self.path().last() {
            trav.graph().expect_child_at(next)
        } else {
            self.pattern_root_child()
        }
    }
}
impl<R: PathRole> PathChild<R> for ChildPath<R> {
    fn path_child<
        'a: 'g,
        'g,
        T: Tokenize,
        Trav: Traversable<T>,
    >(&self, _trav: &'a Trav) -> Child {
        //trav.graph().expect_child_at(R::top_down_iter(self.path.iter()).next().unwrap())
        self.get_child()
    }
}
impl<R: PathRole> PathChild<R> for SearchPath
    where SearchPath: HasRolePath<R>
{
    fn path_child<
        'a: 'g,
        'g,
        T: Tokenize,
        Trav: Traversable<T>,
    >(&self, trav: &'a Trav) -> Child {
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
impl<R, P: PathChild<R>> PathChild<R> for OriginPath<P> {
    fn path_child<
        'a: 'g,
        'g,
        T: Tokenize,
        Trav: Traversable<T>,
    >(&self, trav: &'a Trav) -> Child {
        self.postfix.path_child(trav)
    }
}