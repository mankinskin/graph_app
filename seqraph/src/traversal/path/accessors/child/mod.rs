use crate::*;

pub mod pos;
pub use pos::*;

pub trait DirectChild<R>: ChildPos<R> {
    fn get_direct_child<
        'a: 'g,
        'g,
        T: Tokenize,
        Trav: Traversable<T>,
    >(&self, trav: &'a Trav) -> Child;
}
macro_rules! impl_child {
    {
        DirectChild for $target:ty, $self_:ident, $trav:ident => $func:expr
    } => {
        impl<R> DirectChild<R> for $target
            where $target: ChildPos<R>
        {
            fn get_direct_child<
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
impl_child! { DirectChild for QueryRangePath, self, trav => self.get_child() }
impl_child! { DirectChild for PrefixQuery, self, trav => self.get_child() }
impl DirectChild<End> for OverlapPrimer {
    fn get_direct_child<
        'a: 'g,
        'g,
        T: Tokenize,
        Trav: Traversable<T>
    >(&self, trav: &Trav) -> Child {
        self.get_child()
    }
}
impl DirectChild<Start> for SearchPath {
    fn get_direct_child<
        'a: 'g,
        'g,
        T: Tokenize,
        Trav: Traversable<T>
    >(&self, trav: &Trav) -> Child {
        DirectChild::<Start>::get_direct_child(&self.start, trav)
    }
}
impl DirectChild<End> for SearchPath {
    fn get_direct_child<
        'a: 'g,
        'g,
        T: Tokenize,
        Trav: Traversable<T>
    >(&self, trav: &Trav) -> Child {
        DirectChild::<End>::get_direct_child(&self.end, trav)
    }
}
impl_child! { DirectChild for PathLeaf, self, trav => self.get_child(trav) }
impl_child! { DirectChild for ChildPath, self, trav => self.get_child(trav) }

impl<R, P: DirectChild<R>> DirectChild<R> for OriginPath<P> {
    fn get_direct_child<
        'a: 'g,
        'g,
        T: Tokenize,
        Trav: Traversable<T>,
    >(&self, trav: &Trav) -> Child {
        self.postfix.get_direct_child(trav)
    }
}

/// used to get a direct child in a Graph
pub trait GraphChild<R>: GraphRoot + ChildPos<R> + Send + Sync {
    fn child_location(&self) -> ChildLocation;
    fn get_child<
        'a: 'g,
        'g,
        T: Tokenize,
        Trav: Traversable<T>,
    >(&self, trav: &'a Trav) -> Child {
        trav.graph().expect_child_at(self.child_location())
    }
}
impl<R, P: GraphChild<R>> GraphChild<R> for OriginPath<P> {
    fn child_location(&self) -> ChildLocation {
        self.postfix.child_location()
    }
}
impl GraphChild<Start> for SearchPath {
    fn child_location(&self) -> ChildLocation {
        self.start.child_location()
    }
}
impl GraphChild<End> for SearchPath {
    fn child_location(&self) -> ChildLocation {
        self.end.child_location()
    }
}
impl<R> GraphChild<R> for ChildPath {
    fn child_location(&self) -> ChildLocation {
        match self {
            Self::Path { entry, .. } |
            Self::Leaf(PathLeaf { entry, .. })
                => *entry,
        }
    }
}
impl<R> GraphChild<R> for PathLeaf {
    fn child_location(&self) -> ChildLocation {
        self.entry
    }
}

/// used to get a direct child of a pattern
pub trait PatternChild<R>: ChildPos<R> + PatternRoot {
    fn get_child(&self) -> Child {
        PatternRoot::pattern(self)[self.child_pos()]
    }
}
impl<R> PatternChild<R> for QueryRangePath
    where QueryRangePath: ChildPos<R>
{
}
impl<R> PatternChild<R> for PrefixQuery
    where PrefixQuery: ChildPos<R>
{
}
impl PatternChild<End> for OverlapPrimer {
}

pub struct Start;
pub struct End;


/// used to get a descendant in a Graph, pointed to by a child path
pub trait Descendant<R>: HasPath<R> {
    fn get_descendant_location(&self) -> Option<ChildLocation> {
        // todo: leaf direction
        self.path().last().copied()
    }
    fn get_descendant<
        'a: 'g,
        'g,
        T: Tokenize,
        Trav: Traversable<T>,
    >(&self, trav: &'a Trav) -> Option<Child>;
    //{
    //    trav.graph().get_child_at(self.get_child_location()).ok()
    //}
}
impl<R> Descendant<R> for QueryRangePath
    where QueryRangePath: HasPath<R> + PatternChild<R>
{
    fn get_descendant<
        'a: 'g,
        'g,
        T: Tokenize,
        Trav: Traversable<T>,
    >(&self, trav: &'a Trav) -> Child {
        //if let Some(next) = self.path().last() {
        //    trav.graph().expect_child_at(next)
        //} else {
        //    self.get_child()
        //}
        self.get_child(trav)
    }
}
impl<R> Descendant<R> for SearchPath
    where SearchPath: HasRootedPath<R>
{
    fn get_descendant<
        'a: 'g,
        'g,
        T: Tokenize,
        Trav: Traversable<T>,
    >(&self, trav: &'a Trav) -> Child {
        self.child_path().get_child()
    }
}
impl Descendant<End> for OverlapPrimer {
    fn get_descendant<
        'a: 'g,
        'g,
        T: Tokenize,
        Trav: Traversable<T>,
    >(&self, trav: &'a Trav) -> Child {
        if self.exit == 0 {
            self.start
        } else {
            self.context.get_descendant(trav)
        }
    }
}
impl Descendant<End> for PrefixQuery {
    fn get_descendant<
        'a: 'g,
        'g,
        T: Tokenize,
        Trav: Traversable<T>,
    >(&self, trav: &'a Trav) -> Child {
        if self.end.is_empty() {
            self.get_child(trav)
        } else {
            self.end.get_descendant(trav)
        }
    }
}
impl<R, P: Descendant<R>> Descendant<R> for OriginPath<P> {
    fn get_descendant<
        'a: 'g,
        'g,
        T: Tokenize,
        Trav: Traversable<T>,
    >(&self, trav: &'a Trav) -> Child {
        self.postfix.get_descendant(trav)
    }
}