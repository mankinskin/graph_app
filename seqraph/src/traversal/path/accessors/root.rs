use crate::*;

pub trait GraphRootPattern: GraphRoot + RootPattern {
    fn root_pattern_location(&self) -> PatternLocation;
    fn graph_root_pattern<
        'a: 'g,
        'g,
        T: Tokenize,
        Trav: Traversable<T> + 'a,
    >(&self, trav: &'g Trav::Guard<'a>) -> &'g Pattern {
        trav.expect_pattern_at(self.root_pattern_location())
    }
}
pub trait GraphRoot {
    fn root_parent(&self) -> Child;
}
impl GraphRoot for FoundPath {
    fn root_parent(&self) -> Child {
        match self {
            Self::Complete(c) => *c,
            Self::Range(p) => p.root_parent(),
            Self::Prefix(p) => p.root_parent(),
            Self::Postfix(p) => p.root_parent(),
        }
    }
}
impl<P: GraphRoot> GraphRoot for OriginPath<P> {
    fn root_parent(&self) -> Child {
        self.postfix.root_parent()
    }
}
//impl<T: GraphRootPattern> GraphRoot for T {
//    fn root_parent(&self) -> Child {
//        self.root_pattern_location().parent
//    }
//}
pub trait PatternRoot: RootPattern {
    fn pattern_root_pattern(&self) -> &Pattern;
}
pub trait RootPattern {
    fn root_pattern<
        'a: 'g,
        'b: 'g,
        'g,
        T: Tokenize,
        Trav: Traversable<T> + 'a
    >(&'b self, trav: &'g Trav::Guard<'a>) -> &'g Pattern;
}
//impl<T: GraphRoot> RootChild for T {
//    fn root_child(&self) -> Child {
//        self.root().parent
//    }
//}
macro_rules! impl_root {
    {
        RootPattern for $target:ty, $self_:ident, $trav:ident => $func:expr
    } => {
        impl RootPattern for $target {
            fn root_pattern<
                'a: 'g,
                'b: 'g,
                'g,
                T: Tokenize,
                Trav: Traversable<T> + 'a
            >(&'b $self_, $trav: &'g Trav::Guard<'a>) -> &'g Pattern {
                $func
            }
        }
    };
    {
        PatternRoot for $target:ty, $self_:ident => $func:expr
    } => {
        impl PatternRoot for $target {
            fn pattern_root_pattern(& $self_) -> &Pattern {
                $func
            }
        }
    };
    {
        GraphRootPattern for $target:ty, $self_:ident => $func:expr
    } => {
        impl GraphRootPattern for $target {
            fn root_pattern_location(& $self_) -> PatternLocation {
                $func
            }
        }
    };
    {
        GraphRoot for $target:ty, $self_:ident => $func:expr
    } => {
        impl GraphRoot for $target {
            fn root_parent(& $self_) -> Child {
                $func
            }
        }
    }
    //{
    //    $(< $( $par:ident $( : $bhead:tt $( + $btail:tt )*)? ),* >)? RootChild for $target:ty, $self_:ident => $func:expr
    //} => {
    //    impl $(< $( $par $(: $bhead $( + $btail )* )? ),* >)? RootChild for $target {
    //        fn root_child(& $self_) -> Child {
    //            $func
    //        }
    //    }
    //}
}
impl_root! { PatternRoot for QueryRangePath, self => self.query.borrow() }
//impl_root! { PatternRoot for PrefixQuery, self => self.pattern.borrow() }
//impl_root! { PatternRoot for OverlapPrimer, self => PatternRoot::pattern_root_pattern(&self.context) }

//impl_root! { RootChild for FoundPath, self => 
//    match self {
//        Self::Range(path) => path.root_child(),
//        Self::Postfix(path) => path.root_child(),
//        Self::Prefix(path) => path.root_child(),
//        Self::Complete(c) => *c,
//    }
//}
//impl_root! { <P: RootChild> RootChild for OriginPath<P>, self => 
//    self.postfix.root_child()
//}
//impl_root! { <P: MatchEndPath> RootChild for MatchEnd<P>, self => 
//    match self {
//        MatchEnd::Path(start) => start.root_child(),
//        MatchEnd::Complete(c) => *c,
//    }
//}
//impl_root! { RootChild for SearchPath, self => self.start.root_child() }
//impl_root! { RootChild for ChildPath, self => self.child_location().parent }
//impl_root! { RootChild for PathLeaf, self => self.child_location().parent }

impl_root! { GraphRootPattern for SearchPath, self => self.start.root_pattern_location() }
//impl_root! { GraphRootPattern for ChildPath, self => self.child_location().into_pattern_location() }
//impl_root! { GraphRootPattern for PathLeaf, self => self.child_location().into_pattern_location() }
impl_root! { GraphRoot for SearchPath, self => self.root_pattern_location().parent }

impl<R: ResultKind, Q: BaseQuery> GraphRoot for TraversalNode<R, Q> {
    fn root_parent(&self) -> Child {
        match self {
            Self::Parent(_, node) => node.path.root_parent(),
            Self::Child(_, node) => node.paths.get_path().root_parent(),
            Self::MatchEnd(_, _, path, _) => path.root_parent(),
            Self::Mismatch(_, _, found)
            | Self::QueryEnd(_, _, found) => found.path.root_parent(),
            Self::Start(node) => node.index,
        }
    }
}
impl<P: MatchEndPath + GraphRoot> GraphRoot for MatchEnd<P> {
    fn root_parent(&self) -> Child {
        match self {
            Self::Complete(c) => *c,
            Self::Path(path) => path.root_parent(),
        }
    }
}
impl<R: PathRole> GraphRoot for ChildPath<R> {
    fn root_parent(&self) -> Child {
        self.child_location().parent
    }
}
impl<R: PathRole> GraphRootPattern for ChildPath<R> {
    fn root_pattern_location(&self) -> PatternLocation {
        self.child_location().into_pattern_location()
    }
}
impl<P: GraphRootPattern> GraphRootPattern for OriginPath<P> {
    fn root_pattern_location(&self) -> PatternLocation {
        self.postfix.root_pattern_location()
    }
}

//impl_root! { RootPattern for OverlapPrimer, self, _trav => PatternRoot::pattern_root_pattern(self) }
//impl_root! { RootPattern for PrefixQuery, self, _trav => PatternRoot::pattern_root_pattern(self) }
impl_root! { RootPattern for QueryRangePath, self, _trav => PatternRoot::pattern_root_pattern(self) }

impl_root! { RootPattern for SearchPath, self, trav => GraphRootPattern::graph_root_pattern::<_, Trav>(self, trav) }
//impl_root! { RootPattern for ChildPath, self, trav => GraphRoot::graph_root_pattern(self, trav).borrow() }
//impl_root! { RootPattern for PathLeaf, self, trav => GraphRoot::graph_root_pattern(self, trav).borrow() }
impl<R: PathRole> RootPattern for ChildPath<R> {
    fn root_pattern<
        'a: 'g,
        'b: 'g,
        'g,
        T: Tokenize,
        Trav: Traversable<T> + 'a
    >(&'b self, trav: &'g Trav::Guard<'a>) -> &'g Pattern {
        GraphRootPattern::graph_root_pattern::<_, Trav>(self, trav).borrow()
    }
}

impl<P: RootPattern> RootPattern for OriginPath<P> {
    fn root_pattern<
        'a: 'g,
        'b: 'g,
        'g,
        T: Tokenize,
        Trav: Traversable<T> + 'a
    >(&'b self, trav: &'g Trav::Guard<'a>) -> &'g Pattern {
        self.postfix.root_pattern::<_, Trav>(trav)
    }
}