use crate::*;

// pop path segments
pub trait PathPop {
    fn path_pop(&mut self) -> Option<ChildLocation>;
}

impl<R: PathRole> PathPop for RolePath<R> {
    fn path_pop(&mut self) -> Option<ChildLocation> {
        self.sub_path.path.pop()
    }
}
impl PathPop for SearchPath {
    fn path_pop(&mut self) -> Option<ChildLocation> {
        self.end.path_pop()
    }
}
impl PathPop for QueryRangePath {
    fn path_pop(&mut self) -> Option<ChildLocation> {
        self.end.path_pop()
    }
}
impl PathPop for CachedQuery<'_> {
    fn path_pop(&mut self) -> Option<ChildLocation> {
        self.state.end.path_pop()
    }
}
//impl<P: MatchEndPath + PathPop<Result=Self>> PathPop for MatchEnd<P> {
//    type Result = Result<Self, Child>;
//    fn path_pop<
//        T: Tokenize,
//        D: MatchDirection,
//        Trav: Traversable<T>
//    >(self, trav: &Trav) -> (ChildLocation, Self::Result) {
//        match self {
//            MatchEnd::Path(path) => Ok(path.path_pop::<_, D, _>(trav)),
//            MatchEnd::Complete(child) => Err(child),
//        }
//    }
//}

//impl PathPop for OriginPath<SearchPath> {
//    type Result = OriginPath<<SearchPath as PathPop>::Result>;
//    fn path_pop<
//        T: Tokenize,
//        D: MatchDirection,
//        Trav: Traversable<T>
//    >(self, trav: &Trav) -> Self::Result {
//        OriginPath {
//            postfix: self.postfix.path_pop::<_, D, _>(trav),
//            origin: self.origin.path_pop::<_, D, _>(trav)
//                .unwrap_or_else(|err| MatchEnd::Complete(err))
//        }
//    }
//}