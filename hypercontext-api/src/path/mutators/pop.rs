use crate::{
    graph::vertex::location::child::ChildLocation,
    path::{
        accessors::role::PathRole,
        structs::{
            query_range_path::FoldablePath,
            role_path::RolePath,
            rooted::{
                role_path::RootedRolePath,
                root::PathRoot,
            },
        },
    },
    traversal::state::cursor::PathCursor,
    //traversal::state::query::QueryState,
};

// pop path segments
pub trait PathPop {
    fn path_pop(&mut self) -> Option<ChildLocation>;
}

impl<Role: PathRole, Root: PathRoot> PathPop for RootedRolePath<Role, Root> {
    fn path_pop(&mut self) -> Option<ChildLocation> {
        self.role_path.path_pop()
    }
}

impl<R: PathRole> PathPop for RolePath<R> {
    fn path_pop(&mut self) -> Option<ChildLocation> {
        self.sub_path.path.pop()
    }
}

//impl PathPop for IndexRangePath {
//    fn path_pop(&mut self) -> Option<ChildLocation> {
//        self.end.path_pop()
//    }
//}

impl<P: FoldablePath> PathPop for PathCursor<P> {
    fn path_pop(&mut self) -> Option<ChildLocation> {
        self.path.path_pop()
    }
}

//impl PathPop for QueryState {
//    fn path_pop(&mut self) -> Option<ChildLocation> {
//        self.path.path_pop()
//    }
//}
//impl<P: MatchEndPath + PathPop<Result=Self>> PathPop for MatchEnd<P> {
//    type Result = Result<Self, Child>;
//    fn path_pop<
//        T: Tokenize,
//        D: ,
//        Trav: Traversable<T>
//    >(self, trav: &Trav) -> (ChildLocation, Self::Result) {
//        match self {
//            MatchEnd::Path(path) => Ok(path.path_pop::<_, D, _>(trav)),
//            MatchEnd::Complete(child) => Err(child),
//        }
//    }
//}

//impl PathPop for OriginPath<IndexRangePath> {
//    type Result = OriginPath<<IndexRangePath as PathPop>::Result>;
//    fn path_pop<
//        T: Tokenize,
//        D: ,
//        Trav: Traversable<T>
//    >(self, trav: &Trav) -> Self::Result {
//        OriginPath {
//            postfix: self.postfix.path_pop::<_, D, _>(trav),
//            origin: self.origin.path_pop::<_, D, _>(trav)
//                .unwrap_or_else(|err| MatchEnd::Complete(err))
//        }
//    }
//}
