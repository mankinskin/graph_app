use crate::*;
use super::*;

//pub trait NotStartPath {}
//impl NotStartPath for PathLeaf {}

pub trait MatchEndPath:
    NodePath<Start>
    //+ PathComplete
    //+ PathAppend
    + Into<RootedRolePath<Start>>
    + From<RootedRolePath<Start>>
    //+ From<PathLeaf>
    //+ Into<FoundPath>
    //+ GetCacheKey
    + BasePath
    + Hash
    {}

impl<T:
    NodePath<Start>
    //+ PathComplete
    //+ PathAppend
    + Into<RootedRolePath<Start>>
    + From<RootedRolePath<Start>>
    //+ From<PathLeaf>
    //+ Into<FoundPath>
    //+ GetCacheKey
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
    fn into_mesp(self) -> MatchEnd<RootedRolePath<Start>>;
}
impl<P: MatchEndPath> IntoMatchEndStartPath for MatchEnd<P> {
    fn into_mesp(self) -> MatchEnd<RootedRolePath<Start>> {
        match self {
            MatchEnd::Path(p) => MatchEnd::Path(p.into()),
            MatchEnd::Complete(c) => MatchEnd::Complete(c)
        }
    }
}
//impl<P: MatchEndPath> IntoMatchEndStartPath for OriginPath<MatchEnd<P>> {
//    fn into_mesp(self) -> MatchEnd<RolePath<Start>> {
//        self.postfix.into_mesp()
//    }
//}
//impl From<OriginPath<MatchEnd<RolePath<Start>>>> for MatchEnd<RolePath<Start>> {
//    fn from(start: OriginPath<MatchEnd<RolePath<Start>>>) -> Self {
//        start.postfix
//    }
//}
impl<P: MatchEndPath + From<Q>, Q: Into<RootedRolePath<Start>>> From<Q> for MatchEnd<P> {
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
    //pub fn into_result<R: ResultKind>(self, start: RolePath) -> R::Result<P> {
    //    match self {
    //        Self::Path(start) => Some(start),
    //        _ => None,
    //    }
    //}
}
