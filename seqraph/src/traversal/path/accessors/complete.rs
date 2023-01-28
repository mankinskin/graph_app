use crate::*;


pub trait PathComplete: Sized + Debug {
    //fn new_complete(c: Child) -> Self;
    fn into_complete(&self) -> Option<Child>;

    fn is_complete(&self) -> bool {
        self.into_complete().is_some()
    }
    #[track_caller]
    fn unwrap_complete(self) -> Child {
        self.into_complete()
            .expect(&format!("Unable to unwrap {:?} as complete.", self))
    }
    #[track_caller]
    fn expect_complete(self, msg: &str) -> Child {
        self.into_complete()
            .expect(&format!("Unable to unwrap {:?} as complete: {}", self, msg))
    }
}
impl<R: ResultKind> PathComplete for FoundPath<R> {
    /// returns child if reduced to single child
    fn into_complete(&self) -> Option<Child> {
        match self {
            Self::Complete(c) => Some(*c),
            _ => None,
        }
    }
}
//impl<R: PathRole> PathComplete for RolePath<R> {
//    /// returns child if reduced to single child
//    fn into_complete(&self) -> Option<Child> {
//        self.path.is_empty().then(||
//            self.child
//        )
//    }
//}
impl<P: MatchEndPath> PathComplete for MatchEnd<P> {
    fn into_complete(&self) -> Option<Child> {
        match self {
            Self::Complete(c) => Some(*c),
            _ => None,
        }
    }
}
//impl<P: PathComplete> PathComplete for OriginPath<P> {
//    fn into_complete(&self) -> Option<Child> {
//        self.postfix.into_complete()
//    }
//}

//impl PathComplete for SearchPath {
//    fn is_complete<
//        T: Tokenize,
//        D: MatchDirection,
//        Trav: Traversable<T>,
//    >(&self, trav: &Trav) -> bool {
//        let graph = trav.graph();
//        let pattern = self.root_pattern::<_, Trav>(&graph);
//        <_ as PathBorder<D, _>>::is_complete_in_pattern(&self.start, pattern.borrow()) &&
//            <_ as PathBorder<D, _>>::is_complete_in_pattern(&self.end, pattern.borrow())
//    }
//    fn into_complete<
//        T: Tokenize,
//        D: MatchDirection,
//        Trav: Traversable<T>,
//    >(&self, trav: &Trav) -> Option<Child> {
//        self.is_complete::<_, D, _>(trav).then(||
//            self.root_parent()
//        )
//    }
//}

//impl PathComplete for PathLeaf {
//    /// returns child if reduced to single child
//    fn into_complete<
//        'a: 'g,
//        'g,
//        T: Tokenize,
//        D: MatchDirection,
//        Trav: Traversable<T>,
//    >(&self, trav: &'a Trav) -> Option<Child> {
//        let graph = trav.graph();
//        let pattern = graph.expect_pattern_at(self.entry);
//        (self.entry.sub_index == D::head_index(pattern.borrow()))
//            .then(|| self.entry.parent)
//    }
//}
