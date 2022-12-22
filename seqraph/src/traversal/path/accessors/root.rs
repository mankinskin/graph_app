use crate::*;

pub trait GraphRoot: Root + RootChild {
    fn root(&self) -> PatternLocation;
    fn pattern<
        'a: 'g,
        'g,
        T: Tokenize,
        Trav: Traversable<T>,
    >(&self, trav: &'a Trav) -> Pattern {
        trav.graph().expect_pattern_at(self.root())
    }
}
pub trait PatternRoot: Root {
    fn pattern(&self) -> &[Child];
}
pub trait Root {
    fn pattern<
        'a: 'g,
        'g,
        T: Tokenize,
        Trav: Traversable<T>
    >(&self, trav: &Trav) -> &[Child];
}
pub trait RootChild {
    fn root_child(&self) -> Child;
}
//impl<T: GraphRoot> RootChild for T {
//    fn root_child(&self) -> Child {
//        self.root().parent
//    }
//}
macro_rules! impl_root {
    {
        Root for $target:ty, $self_:ident, $trav:ident => $func:expr
    } => {
        impl Root for $target {
            fn pattern<
                'a: 'g,
                'g,
                T: Tokenize,
                Trav: Traversable<T>
            >(& $self_, $trav: &Trav) -> &[Child] {
                $func
            }
        }
    };
    {
        PatternRoot for $target:ty, $self_:ident => $func:expr
    } => {
        impl PatternRoot for $target {
            fn pattern(& $self_) -> &[Child] {
                $func
            }
        }
    };
    {
        GraphRoot for $target:ty, $self_:ident => $func:expr
    } => {
        impl GraphRoot for $target {
            fn root(& $self_) -> PatternLocation {
                $func
            }
        }
    };
    {
        $(< $( $par:ident $( : $bhead:tt $( + $btail:tt )*)? ),* >)? RootChild for $target:ty, $self_:ident => $func:expr
    } => {
        impl $(< $( $par $(: $bhead $( + $btail )* )? ),* >)? RootChild for $target {
            fn root_child(& $self_) -> Child {
                $func
            }
        }
    }
}
impl_root! { PatternRoot for QueryRangePath, self => self.query.borrow() }
impl_root! { PatternRoot for PrefixQuery, self => self.pattern.borrow() }
impl_root! { PatternRoot for OverlapPrimer, self => PatternRoot::pattern(&self.context) }

impl_root! { RootChild for FoundPath, self => 
    match self {
        Self::Range(path) => path.root_child(),
        Self::Postfix(path) => path.root_child(),
        Self::Prefix(path) => path.root_child(),
        Self::Complete(c) => *c,
    }
}
impl_root! { <P: RootChild> RootChild for OriginPath<P>, self => 
    self.postfix.root_child()
}
impl_root! { <P: MatchEndPath> RootChild for MatchEnd<P>, self => 
    match self {
        MatchEnd::Path(start) => start.root_child(),
        MatchEnd::Complete(c) => *c,
    }
}
impl_root! { RootChild for SearchPath, self => self.start.root_child() }
impl_root! { RootChild for ChildPath, self => self.child_location().parent }
impl_root! { RootChild for PathLeaf, self => self.child_location().parent }

impl_root! { GraphRoot for SearchPath, self => self.start.root() }
impl_root! { GraphRoot for ChildPath, self => self.child_location().into_pattern_location() }
impl_root! { GraphRoot for PathLeaf, self => self.child_location().into_pattern_location() }

impl<P: GraphRoot> GraphRoot for OriginPath<P> {
    fn root(&self) -> PatternLocation {
        self.postfix.root()
    }
}

impl_root! { Root for OverlapPrimer, self, _trav => PatternRoot::pattern(self) }
impl_root! { Root for PrefixQuery, self, _trav => PatternRoot::pattern(self) }
impl_root! { Root for QueryRangePath, self, _trav => PatternRoot::pattern(self) }

impl_root! { Root for SearchPath, self, trav => GraphRoot::pattern(self, trav).borrow() }
impl_root! { Root for ChildPath, self, trav => GraphRoot::pattern(self, trav).borrow() }
impl_root! { Root for PathLeaf, self, trav => GraphRoot::pattern(self, trav).borrow() }

impl<P: Root> Root for OriginPath<P> {
    fn pattern<
        'a: 'g,
        'g,
        T: Tokenize,
        Trav: Traversable<T>
    >(&self, trav: &Trav) -> &[Child] {
        self.postfix.pattern(trav)
    }
}