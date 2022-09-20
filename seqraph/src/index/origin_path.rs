use crate::*;

#[derive(Clone, PartialEq, Eq)]
pub(crate) struct OriginPath<P: MatchEndPath> {
    pub(crate) match_end: MatchEnd<P>,
    pub(crate) origin: StartPath,
}

impl<P: MatchEndPath> From<P> for OriginPath<P> {
    fn from(start: P) -> Self {
        OriginPath {
            match_end: MatchEnd::from(start.clone()),
            origin: start.into(),
        }
    }
}
impl<P: MatchEndPath> RootChild for OriginPath<P> {
    fn root_child(&self) -> Child {
        self.match_end.root_child()
    }
}
impl<P: MatchEndPath> PathComplete for OriginPath<P> {
    fn complete<
        'a: 'g,
        'g,
        T: Tokenize,
        D: MatchDirection,
        Trav: Traversable<'a, 'g, T>,
    >(&self, trav: &'a Trav) -> Option<Child> {
        self.match_end.complete::<_, D, _>(trav)
    }
}
impl<P: MatchEndPath> PathReduce for OriginPath<P> {
    fn reduce<
        'a: 'g,
        'g,
        T: Tokenize,
        D: MatchDirection,
        Trav: Traversable<'a, 'g, T>,
    >(&mut self, trav: &'a Trav) {
        self.match_end.reduce::<_, D, _>(trav)
    }
}
impl<P: MatchEndPath> PathAppend for OriginPath<P> {
    type Result = StartPath;
    fn append<
        'a: 'g,
        'g,
        T: Tokenize,
        D: MatchDirection,
        Trav: Traversable<'a, 'g, T>
    >(self, trav: &'a Trav, parent_entry: ChildLocation) -> Self::Result {
        self.origin.append::<_, D, _>(trav, parent_entry);
        self.match_end.append::<_, D, _>(trav, parent_entry)
    }
}