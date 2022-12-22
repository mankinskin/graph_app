use crate::*;


pub trait PathComplete: Send + Sync {
    //fn new_complete(c: Child) -> Self;
    fn complete<
        'a: 'g,
        'g,
        T: Tokenize,
        D: MatchDirection,
        Trav: Traversable<T>,
    >(&self, trav: &'a Trav) -> Option<Child>;

    fn is_complete<
        'a: 'g,
        'g,
        T: Tokenize,
        D: MatchDirection,
        Trav: Traversable<T>,
    >(&self, trav: &'a Trav) -> bool {
        self.complete::<_, D, _>(trav).is_some()
    }
}


impl<P: PathComplete> PathComplete for OriginPath<P> {
    fn complete<
        'a: 'g,
        'g,
        T: Tokenize,
        D: MatchDirection,
        Trav: Traversable<T>,
    >(&self, trav: &'a Trav) -> Option<Child> {
        self.postfix.complete::<_, D, _>(trav)
    }
}

impl PathComplete for SearchPath {
    fn is_complete<
        'a: 'g,
        'g,
        T: Tokenize,
        D: MatchDirection,
        Trav: Traversable<T>,
    >(&self, trav: &'a Trav) -> bool {
        let pattern = self.get_pattern(trav);
        <ChildPath as PathBorder<D>>::pattern_is_complete(self.child_path(), &pattern[..]) &&
            self.child_path().is_empty() &&
            <ChildPath as PathBorder<D>>::pattern_entry_outer_pos(pattern, self.child_pos()).is_none()
    }
    fn complete<
        'a: 'g,
        'g,
        T: Tokenize,
        D: MatchDirection,
        Trav: Traversable<T>,
    >(&self, trav: &'a Trav) -> Option<Child> {
        self.is_complete::<_, D, _>(trav).then(||
            self.root_child()
        )
    }
}

impl PathComplete for PathLeaf {
    /// returns child if reduced to single child
    fn complete<
        'a: 'g,
        'g,
        T: Tokenize,
        D: MatchDirection,
        Trav: Traversable<T>,
    >(&self, trav: &'a Trav) -> Option<Child> {
        let graph = trav.graph();
        let pattern = graph.expect_pattern_at(self.entry);
        (self.entry.sub_index == D::head_index(pattern.borrow()))
            .then(|| self.entry.parent)
    }
}

impl PathComplete for ChildPath {
    /// returns child if reduced to single child
    fn complete<
        'a: 'g,
        'g,
        T: Tokenize,
        D: MatchDirection,
        Trav: Traversable<T>,
    >(&self, trav: &'a Trav) -> Option<Child> {
        match self {
            Self::Leaf(leaf) => leaf.complete::<_, D, _>(trav),
            // TODO: maybe skip path segments starting at pattern head
            Self::Path { .. } => None,
        }
    }
}