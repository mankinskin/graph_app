use crate::{
    graph::vertex::{
        child::Child,
        location::pattern::PatternLocation,
        pattern::Pattern,
    },
    path::structs::match_end::{
        MatchEnd,
        MatchEndPath,
    },
    traversal::{
        state::end::{
            EndKind,
            EndState,
        },
        traversable::Traversable,
    },
};

pub trait GraphRootPattern: GraphRoot + RootPattern {
    fn root_pattern_location(&self) -> PatternLocation;
    fn graph_root_pattern<'a: 'g, 'g, Trav: Traversable + 'a>(
        &self,
        trav: &'g Trav::Guard<'a>,
    ) -> &'g Pattern {
        trav.expect_pattern_at(self.root_pattern_location())
    }
}

pub trait GraphRoot {
    fn root_parent(&self) -> Child;
}

//impl GraphRoot for FoundPath {
//    fn root_parent(&self) -> Child {
//        match self {
//            Self::Complete(c) => *c,
//            Self::Path(p) => p.root_parent(),
//            Self::Prefix(p) => p.root_parent(),
//            Self::Postfix(p) => p.root_parent(),
//        }
//    }
//}
//impl<P: GraphRoot> GraphRoot for OriginPath<P> {
//    fn root_parent(&self) -> Child {
//        self.postfix.root_parent()
//    }
//}
//impl<T: GraphRootPattern> GraphRoot for T {
//    fn root_parent(&self) -> Child {
//        self.root_pattern_location().parent
//    }
//}
pub trait PatternRoot {
    fn pattern_root_pattern(&self) -> &Pattern;
}

pub trait RootPattern {
    fn root_pattern<'a: 'g, 'b: 'g, 'g, Trav: Traversable + 'a>(
        &'b self,
        trav: &'g Trav::Guard<'a>,
    ) -> &'g Pattern;
}
//impl<T: GraphRoot> RootChild for T {
//    fn root_child(&self) -> Child {
//        self.root().parent
//    }
//}
#[macro_export]
macro_rules! impl_root {
    {
        RootPattern for $target:ty, $self_:ident, $trav:ident => $func:expr
    } => {
        impl $crate::path::accessors::root::RootPattern for $target {
            fn root_pattern<
                'a: 'g,
                'b: 'g,
                'g,
                Trav: $crate::traversal::traversable::Traversable + 'a
            >(&'b $self_, $trav: &'g Trav::Guard<'a>) -> &'g $crate::graph::vertex::pattern::Pattern {
                $func
            }
        }
    };
    {
        PatternRoot for $target:ty, $self_:ident => $func:expr
    } => {
        impl $crate::path::accessors::root::PatternRoot for $target {
            fn pattern_root_pattern(& $self_) -> &Pattern {
                $func
            }
        }
    };
    {
        GraphRootPattern for $target:ty, $self_:ident => $func:expr
    } => {
        impl GraphRootPattern for $target {
            fn root_pattern_location(& $self_) -> $crate::graph::vertex::location::pattern::PatternLocation {
                $func
            }
        }
    };
    {
        GraphRoot for $target:ty, $self_:ident => $func:expr
    } => {
        impl $crate::path::accessors::root::GraphRoot for $target {
            fn root_parent(& $self_) -> $crate::graph::vertex::child::Child {
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

//impl_root! { PatternRoot for OverlapPrimer, self => PatternRoot::pattern_root_pattern(&self.context) }
//impl_root! { PatternRoot for QueryState, self => self.path.root.borrow() }
//impl PatternRoot for QueryState {
//    fn pattern_root_pattern(&self) -> &Pattern {
//        &self.path.root
//    }
//}
//impl_root! { RootChild for FoundPath, self =>
//    match self {
//        Self::Path(path) => path.root_child(),
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
//impl_root! { RootChild for IndexRangePath, self => self.start.root_child() }
//impl_root! { RootChild for RolePath, self => self.child_location().parent }
//impl_root! { RootChild for PathLeaf, self => self.child_location().parent }

//impl_root! { GraphRootPattern for RolePath, self => self.child_location().into_pattern_location() }
//impl_root! { GraphRootPattern for PathLeaf, self => self.child_location().into_pattern_location() }
impl_root! { GraphRoot for EndState, self =>
    match &self.kind {
        EndKind::Complete(c) => *c,
        EndKind::Range(p) => p.path.root_parent(),
        EndKind::Postfix(p) => p.path.root_parent(),
        EndKind::Prefix(p) => p.path.root_parent(),
    }
}

//impl<R: ResultKind, Q: BaseQuery> GraphRoot for TraversalState<R, Q> {
//    fn root_parent(&self) -> Child {
//        match self {
//            Self::Parent(_, node) => node.path.root_parent(),
//            Self::Child(_, node) => node.paths.get_path().root_parent(),
//            Self::End(_, state) => state.root_parent(),
//            Self::Start(node) => node.index,
//        }
//    }
//}
//impl<R: ResultKind, Q: BaseQuery> GraphRoot for EndState<R, Q> {
//    fn root_parent(&self) -> Child {
//        match self {
//            Self::MatchEnd(_, path, _) => path.root_parent(),
//            Self::Mismatch(_, _, _, found)
//            | Self::QueryEnd(_, _, _, found) => found.path.root_parent(),
//        }
//    }
//}
impl<P: MatchEndPath + GraphRoot> GraphRoot for MatchEnd<P> {
    fn root_parent(&self) -> Child {
        match self {
            Self::Complete(c) => *c,
            Self::Path(path) => path.root_parent(),
        }
    }
}

//impl<P: GraphRootPattern> GraphRootPattern for OriginPath<P> {
//    fn root_pattern_location(&self) -> PatternLocation {
//        self.postfix.root_pattern_location()
//    }
//}

//impl_root! { RootPattern for OverlapPrimer, self, _trav => PatternRoot::pattern_root_pattern(self) }
//impl_root! { RootPattern for PatternPrefixPath, self, _trav => PatternRoot::pattern_root_pattern(self) }

//impl_root! { RootPattern for RolePath, self, trav => GraphRoot::graph_root_pattern(self, trav).borrow() }
//impl_root! { RootPattern for PathLeaf, self, trav => GraphRoot::graph_root_pattern(self, trav).borrow() }

//impl<P: RootPattern> RootPattern for OriginPath<P> {
//    fn root_pattern<
//        'a: 'g,
//        'b: 'g,
//        'g,
//        T: Tokenize,
//        Trav: Traversable<T> + 'a
//    >(&'b self, trav: &'g Trav::Guard<'a>) -> &'g Pattern {
//        self.postfix.root_pattern::<_, Trav>(trav)
//    }
//}
