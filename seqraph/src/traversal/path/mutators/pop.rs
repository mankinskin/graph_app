use crate::*;

// pop path segments
pub trait PathPop {
    fn pop_path(&mut self) -> Option<ChildLocation>;
}

impl<R: PathRole> PathPop for RolePath<R> {
    fn pop_path(&mut self) -> Option<ChildLocation> {
        self.sub_path.path.pop()
    }
}
impl PathPop for SearchPath {
    fn pop_path(&mut self) -> Option<ChildLocation> {
        self.end.pop_path()
    }
}
impl PathPop for QueryRangePath {
    fn pop_path(&mut self) -> Option<ChildLocation> {
        self.end.pop_path()
    }
}
//impl<P: MatchEndPath + PathPop<Result=Self>> PathPop for MatchEnd<P> {
//    type Result = Result<Self, Child>;
//    fn pop_path<
//        T: Tokenize,
//        D: MatchDirection,
//        Trav: Traversable<T>
//    >(self, trav: &Trav) -> (ChildLocation, Self::Result) {
//        match self {
//            MatchEnd::Path(path) => Ok(path.pop_path::<_, D, _>(trav)),
//            MatchEnd::Complete(child) => Err(child),
//        }
//    }
//}

//impl PathPop for OriginPath<SearchPath> {
//    type Result = OriginPath<<SearchPath as PathPop>::Result>;
//    fn pop_path<
//        T: Tokenize,
//        D: MatchDirection,
//        Trav: Traversable<T>
//    >(self, trav: &Trav) -> Self::Result {
//        OriginPath {
//            postfix: self.postfix.pop_path::<_, D, _>(trav),
//            origin: self.origin.pop_path::<_, D, _>(trav)
//                .unwrap_or_else(|err| MatchEnd::Complete(err))
//        }
//    }
//}
