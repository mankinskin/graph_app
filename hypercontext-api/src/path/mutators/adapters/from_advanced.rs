use crate::traversal::traversable::Traversable;

use super::Advanced;

pub trait FromAdvanced<A: Advanced> {
    fn from_advanced<Trav: Traversable>(
        path: A,
        trav: &Trav,
    ) -> Self;
}

//impl FromAdvanced<IndexRangePath> for FoundPath {
//    fn from_advanced<
//        T: Tokenize,
//        D: MatchDirection,
//        Trav: Traversable<T>
//    >(path: IndexRangePath, trav: &Trav) -> Self {
//        if path.is_complete::<_, D, _>(trav) {
//            Self::Complete(<IndexRangePath as GraphRootChild<Start>>::root_child_location(&path).parent)
//        } else {
//            Self::Path(path)
//        }
//
//    }
//}
//impl FromAdvanced<OriginPath<IndexRangePath>> for OriginPath<FoundPath> {
//    fn from_advanced<
//        T: Tokenize,
//        D: MatchDirection,
//        Trav: Traversable<T>
//    >(path: OriginPath<IndexRangePath>, trav: &Trav) -> Self {
//        Self {
//            postfix: FoundPath::from_advanced::<_, D, _>(path.postfix, trav),
//            origin: path.origin,
//        }
//    }
//}
//impl FromAdvanced<OriginPath<IndexRangePath>> for OriginPath<RolePath<Start>> {
//    fn from_advanced<
//        T: Tokenize,
//        D: MatchDirection,
//        Trav: Traversable<T>
//    >(path: OriginPath<IndexRangePath>, trav: &Trav) -> Self {
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
