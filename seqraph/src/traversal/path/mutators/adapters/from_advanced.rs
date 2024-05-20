use crate::{
    traversal::{
        path::{
            accessors::role::Start,
            structs::{
                role_path::RolePath,
                rooted_path::SearchPath,
            },
        },
        result_kind::Advanced,
        traversable::Traversable,
    },
};

pub trait FromAdvanced<A: Advanced> {
    fn from_advanced<Trav: Traversable>(
        path: A,
        trav: &Trav,
    ) -> Self;
}
//impl FromAdvanced<SearchPath> for FoundPath {
//    fn from_advanced<
//        T: Tokenize,
//        D: MatchDirection,
//        Trav: Traversable<T>
//    >(path: SearchPath, trav: &Trav) -> Self {
//        if path.is_complete::<_, D, _>(trav) {
//            Self::Complete(<SearchPath as GraphRootChild<Start>>::root_child_location(&path).parent)
//        } else {
//            Self::Path(path)
//        }
//
//    }
//}
impl FromAdvanced<SearchPath> for RolePath<Start> {
    fn from_advanced<Trav: Traversable>(
        path: SearchPath,
        _trav: &Trav,
    ) -> Self {
        path.start
    }
}
//impl FromAdvanced<OriginPath<SearchPath>> for OriginPath<FoundPath> {
//    fn from_advanced<
//        T: Tokenize,
//        D: MatchDirection,
//        Trav: Traversable<T>
//    >(path: OriginPath<SearchPath>, trav: &Trav) -> Self {
//        Self {
//            postfix: FoundPath::from_advanced::<_, D, _>(path.postfix, trav),
//            origin: path.origin,
//        }
//    }
//}
//impl FromAdvanced<OriginPath<SearchPath>> for OriginPath<RolePath<Start>> {
//    fn from_advanced<
//        T: Tokenize,
//        D: MatchDirection,
//        Trav: Traversable<T>
//    >(path: OriginPath<SearchPath>, trav: &Trav) -> Self {
//        Self {
//            postfix: RolePath::from_advanced::<_, D, _>(path.postfix, trav),
//            origin: path.origin,
//        }
//    }
//}
//
//impl<A: Advanced, F: FromAdvanced<A>> FromAdvanced<A> for OriginPath<F> {
//    fn from_advanced<
//        T: Tokenize,
//        D: MatchDirection,
//        Trav: Traversable<T>
//    >(path: A, trav: &Trav) -> Self {
//        Self {
//            origin: MatchEnd::Path(HasRolePath::<Start>::role_path(&path).clone()),
//            postfix: F::from_advanced::<_, D, _>(path, trav),
//        }
//    }
//}
