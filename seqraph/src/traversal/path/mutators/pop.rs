use crate::*;

pub trait PathPop {
    type Result;
    fn pop_path<
        T: Tokenize,
        D: MatchDirection,
        Trav: Traversable<T>
    >(self, trav: &Trav) -> Self::Result;
}

impl PathPop for OriginPath<SearchPath> {
    type Result = OriginPath<<SearchPath as PathPop>::Result>;
    fn pop_path<
        T: Tokenize,
        D: MatchDirection,
        Trav: Traversable<T>
    >(self, trav: &Trav) -> Self::Result {
        OriginPath {
            postfix: self.postfix.pop_path::<_, D, _>(trav),
            origin: self.origin.pop_path::<_, D, _>(trav)
                .unwrap_or_else(|err| MatchEnd::Complete(err))
        }
    }
}

impl PathPop for SearchPath {
    type Result = <ChildPath<Start> as PathPop>::Result;
    fn pop_path<
        T: Tokenize,
        D: MatchDirection,
        Trav: Traversable<T>
    >(self, trav: &Trav) -> Self::Result {
        self.start.pop_path::<_, D, _>(trav)
    }
}

impl PathPop for ChildPath<Start> {
    type Result = MatchEnd<ChildPath<Start>>;
    fn pop_path<
        T: Tokenize,
        D: MatchDirection,
        Trav: Traversable<T>
    >(mut self, _trav: &Trav) -> Self::Result {
        let len = self.path.len();
        if len == 1 {
            MatchEnd::Complete(self.child)
        } else {
            let _ = self.path.pop().unwrap();
            MatchEnd::Path(self)
        }
    }
}
impl<P: MatchEndPath + PathPop<Result=Self>> PathPop for MatchEnd<P> {
    type Result = Result<Self, Child>;
    fn pop_path<
        T: Tokenize,
        D: MatchDirection,
        Trav: Traversable<T>
    >(self, trav: &Trav) -> Self::Result {
        match self {
            MatchEnd::Path(path) => Ok(path.pop_path::<_, D, _>(trav)),
            MatchEnd::Complete(child) => Err(child),
        }
    }
}