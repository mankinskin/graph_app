
#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub struct OriginPath<P> {
    pub postfix: P,
    pub origin: MatchEnd<RolePath<Start>>,
}

impl<P: Into<RolePath<Start>>> From<P> for OriginPath<RolePath<Start>> {
    fn from(start: P) -> Self {
        let origin = start.into();
        OriginPath {
            postfix: origin.clone(),
            origin: MatchEnd::Path(origin),
        }
    }
}
//impl<Q: Into<RolePath>> From<OriginPath<Q>> for RolePath {
//    fn from(start: OriginPath<Q>) -> Self {
//        start.postfix.into()
//    }
//}
impl From<OriginPath<SearchPath>> for OriginPath<RolePath<Start>> {
    fn from(other: OriginPath<SearchPath>) -> Self {
        OriginPath {
            postfix: RolePath::from(other.postfix),
            origin: other.origin,
        }
    }
}
impl From<OriginPath<RolePath<Start>>> for OriginPath<MatchEnd<RolePath<Start>>> {
    fn from(other: OriginPath<RolePath<Start>>) -> Self {
        OriginPath {
            postfix: MatchEnd::from(other.postfix),
            origin: other.origin,
        }
    }
}
//impl From<OriginPath<MatchEnd<RolePath<Start>>>> for OriginPath<FoundPath> {
//    fn from(other: OriginPath<MatchEnd<RolePath<Start>>>) -> Self {
//        OriginPath {
//            postfix: FoundPath::from(other.postfix),
//            origin: other.origin,
//        }
//    }
//}
////impl From<OriginPath<RolePath>> for OriginPath<SearchPath> {
//    fn from(other: OriginPath<RolePath>) -> Self {
//        OriginPath {
//            postfix: SearchPath::from(other.postfix),
//            origin: other.origin,
//        }
//    }
//}
//impl From<OriginPath<PathLeaf>> for OriginPath<RolePath> {
//    fn from(other: OriginPath<PathLeaf>) -> Self {
//        OriginPath {
//            postfix: RolePath::from(other.postfix),
//            origin: other.origin,
//        }
//    }
//}
pub trait Origin {
    fn into_origin(self) -> MatchEnd<RolePath<Start>>;
}
impl<P> Origin for OriginPath<P> {
    fn into_origin(self) -> MatchEnd<RolePath<Start>> {
        self.origin
    }
}

impl<P: Ord> PartialOrd for OriginPath<P> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.postfix.partial_cmp(&other.postfix)
    }
}
impl<P: Ord> Ord for OriginPath<P> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.postfix.cmp(&other.postfix)
    }
}
impl<P: Wide> Wide for OriginPath<P> {
    fn width(&self) -> usize {
        self.postfix.width()
    }
}