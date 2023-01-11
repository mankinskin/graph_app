use crate::*;


pub trait PathComplete {
    //fn new_complete(c: Child) -> Self;
    fn into_complete<
        'a: 'g,
        'g,
        T: Tokenize,
        D: MatchDirection,
        Trav: Traversable<T>,
    >(&'a self, trav: &'a Trav) -> Option<Child>;

    fn is_complete<
        'a: 'g,
        'g,
        T: Tokenize,
        D: MatchDirection,
        Trav: Traversable<T>,
    >(&'a self, trav: &'a Trav) -> bool {
        self.into_complete::<_, D, _>(trav).is_some()
    }
}
impl<P: PathComplete> PathComplete for OriginPath<P> {
    fn into_complete<
        'a: 'g,
        'g,
        T: Tokenize,
        D: MatchDirection,
        Trav: Traversable<T>,
    >(&'a self, trav: &'a Trav) -> Option<Child> {
        self.postfix.into_complete::<_, D, _>(trav)
    }
}

impl PathComplete for SearchPath {
    fn is_complete<
        'a: 'g,
        'g,
        T: Tokenize,
        D: MatchDirection,
        Trav: Traversable<T> + 'a,
    >(&'a self, trav: &'g Trav) -> bool {
        let graph = trav.graph();
        let pattern = self.root_pattern::<_, Trav>(&graph);
        <_ as PathBorder<D, _>>::is_complete_in_pattern(&self.start, pattern.borrow()) &&
            <_ as PathBorder<D, _>>::is_complete_in_pattern(&self.end, pattern.borrow())
    }
    fn into_complete<
        'a: 'g,
        'g,
        T: Tokenize,
        D: MatchDirection,
        Trav: Traversable<T>,
    >(&'a self, trav: &'a Trav) -> Option<Child> {
        self.is_complete::<_, D, _>(trav).then(||
            self.root_parent()
        )
    }
}

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

impl<R> PathComplete for ChildPath<R> {
    /// returns child if reduced to single child
    fn into_complete<
        'a: 'g,
        'g,
        T: Tokenize,
        D: MatchDirection,
        Trav: Traversable<T>,
    >(&self, _trav: &'a Trav) -> Option<Child> {
        self.path.is_empty().then(||
            self.child
        )
    }
}
impl<P: MatchEndPath> PathComplete for MatchEnd<P> {
    fn into_complete<
        'a: 'g,
        'g,
        T: Tokenize,
        D: MatchDirection,
        Trav: Traversable<T>,
    >(&self, _trav: &'a Trav) -> Option<Child> {
        match self {
            Self::Complete(c) => Some(*c),
            _ => None,
        }
    }
}

//impl<P: RangePath> PathComplete for OriginPath<P> {
//    fn into_complete<
//        'a: 'g,
//        'g,
//        T: Tokenize,
//        D: MatchDirection,
//        Trav: Traversable<T>,
//    >(&self, trav: Trav) -> Option<Child> {
//        self.postfix.into_complete::<_, D, _>(trav)
//    }
//}