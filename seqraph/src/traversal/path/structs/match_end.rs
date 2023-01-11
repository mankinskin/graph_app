use crate::*;
use super::*;

//pub trait NotStartPath {}
//impl NotStartPath for PathLeaf {}

pub trait MatchEndPath:
    NodePath<Start>
    + PathComplete
    + PathAppend<Result=ChildPath<Start>>
    + Into<ChildPath<Start>>
    + From<ChildPath<Start>>
    //+ From<PathLeaf>
    + Into<FoundPath>
    + GetCacheKey
    + BasePath
    + Hash
    {}
impl<T:
    NodePath<Start>
    + PathComplete
    + PathAppend<Result=ChildPath<Start>>
    + Into<ChildPath<Start>>
    + From<ChildPath<Start>>
    //+ From<PathLeaf>
    + Into<FoundPath>
    + GetCacheKey
    + Hash
    + BasePath
> MatchEndPath for T {}

/// Used to represent results after traversal with only a start path
#[derive(Clone, Debug, PartialEq, Hash, Eq)]
pub enum MatchEnd<P: MatchEndPath> {
    Path(P),
    Complete(Child),
}
pub trait IntoMatchEndStartPath {
    fn into_mesp(self) -> MatchEnd<ChildPath<Start>>;
}
impl<P: MatchEndPath> RangePath for MatchEnd<P> {
    fn into_complete(self) -> Option<Child> {
        match self {
            Self::Complete(index) => Some(index),
            _ => None,
        }
    }
}
//impl<P: RangePath> Into<FoundPath> for MatchEnd<P> {
//    fn into(self) -> FoundPath {
//        match self {
//            MatchEnd::Path(path) => p.into(),
//            MatchEnd::Complete(c) => FoundPath::Complete(c),
//        }
//    }
//}
//impl<P: RangePath> Into<FoundPath> for OriginPath<P> {
//    fn into(self) -> FoundPath {
//        self.postfix.into()
//    }
//}
//impl<P: MatchEndPath> FromSearchPath for MatchEnd<P> {
//    fn from_search_path<
//        'a: 'g,
//        'g,
//        T: Tokenize,
//        D: MatchDirection,
//        Trav: Traversable<T>
//    >(path: SearchPath, trav: Trav) -> Self {
//        Self {
//            origin: path.role_path().clone(),
//            postfix: P::from_search_path(path, trav),
//        }
//    }
//}
//impl<P: MatchEndPath> FromMatchEnd<P> for OriginPath<MatchEnd<P>> {
//    fn from_match_end(match_end: MatchEnd<P>, origin: ChildPath) -> Self {
//        OriginPath {
//            postfix: match_end,
//            origin,
//        }
//    }
//}
impl<P: MatchEndPath> IntoMatchEndStartPath for MatchEnd<P> {
    fn into_mesp(self) -> MatchEnd<ChildPath<Start>> {
        match self {
            MatchEnd::Path(p) => MatchEnd::Path(p.into()),
            MatchEnd::Complete(c) => MatchEnd::Complete(c)
        }
    }
}
impl<P: MatchEndPath> IntoRangePath for MatchEnd<P> {
    type Result = FoundPath;
    fn into_range_path(self) -> Self::Result {
        FoundPath::from(self.into_mesp())
    }
}
impl<P: MatchEndPath> IntoMatchEndStartPath for OriginPath<MatchEnd<P>> {
    fn into_mesp(self) -> MatchEnd<ChildPath<Start>> {
        self.postfix.into_mesp()
    }
}
impl From<OriginPath<MatchEnd<ChildPath<Start>>>> for MatchEnd<ChildPath<Start>> {
    fn from(start: OriginPath<MatchEnd<ChildPath<Start>>>) -> Self {
        start.postfix
    }
}
//impl From<MatchEnd<PathLeaf>> for MatchEnd<ChildPath> {
//    fn from(start: MatchEnd<PathLeaf>) -> Self {
//        match start {
//            MatchEnd::Path(leaf) => MatchEnd::Path(leaf.into()),
//            MatchEnd::Complete(c) => MatchEnd::Complete(c)
//        }
//    }
//}
impl<P: MatchEndPath + From<Q>, Q: Into<ChildPath<Start>>> From<Q> for MatchEnd<P> {
    fn from(start: Q) -> Self {
        MatchEnd::Path(P::from(start))
    }
}
impl<P: MatchEndPath> MatchEnd<P> {
    #[allow(unused)]
    pub fn unwrap_path(self) -> P {
        match self {
            Self::Path(path) => Some(path),
            _ => None,
        }.unwrap()
    }
    pub fn get_path(&self) -> Option<&P> {
        match self {
            Self::Path(start) => Some(start),
            _ => None,
        }
    }
    //pub fn into_result<R: ResultKind>(self, start: ChildPath) -> R::Result<P> {
    //    match self {
    //        Self::Path(start) => Some(start),
    //        _ => None,
    //    }
    //}
}
