use crate::*;

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub struct OriginPath<P> {
    pub postfix: P,
    pub origin: MatchEnd<ChildPath<Start>>,
}

impl<P: Into<ChildPath<Start>>> From<P> for OriginPath<ChildPath<Start>> {
    fn from(start: P) -> Self {
        let origin = start.into();
        OriginPath {
            postfix: origin.clone(),
            origin: MatchEnd::Path(origin),
        }
    }
}
//impl<Q: Into<ChildPath>> From<OriginPath<Q>> for ChildPath {
//    fn from(start: OriginPath<Q>) -> Self {
//        start.postfix.into()
//    }
//}
impl From<OriginPath<SearchPath>> for OriginPath<ChildPath<Start>> {
    fn from(other: OriginPath<SearchPath>) -> Self {
        OriginPath {
            postfix: ChildPath::from(other.postfix),
            origin: other.origin,
        }
    }
}
impl From<OriginPath<ChildPath<Start>>> for OriginPath<MatchEnd<ChildPath<Start>>> {
    fn from(other: OriginPath<ChildPath<Start>>) -> Self {
        OriginPath {
            postfix: MatchEnd::from(other.postfix),
            origin: other.origin,
        }
    }
}
impl From<OriginPath<MatchEnd<ChildPath<Start>>>> for OriginPath<FoundPath> {
    fn from(other: OriginPath<MatchEnd<ChildPath<Start>>>) -> Self {
        OriginPath {
            postfix: FoundPath::from(other.postfix),
            origin: other.origin,
        }
    }
}
//impl From<OriginPath<ChildPath>> for OriginPath<SearchPath> {
//    fn from(other: OriginPath<ChildPath>) -> Self {
//        OriginPath {
//            postfix: SearchPath::from(other.postfix),
//            origin: other.origin,
//        }
//    }
//}
//impl From<OriginPath<PathLeaf>> for OriginPath<ChildPath> {
//    fn from(other: OriginPath<PathLeaf>) -> Self {
//        OriginPath {
//            postfix: ChildPath::from(other.postfix),
//            origin: other.origin,
//        }
//    }
//}
pub trait Origin {
    fn into_origin(self) -> MatchEnd<ChildPath<Start>>;
}
impl<P> Origin for OriginPath<P> {
    fn into_origin(self) -> MatchEnd<ChildPath<Start>> {
        self.origin
    }
}
impl<R: RangePath> IntoRangePath for OriginPath<R> {
    type Result = OriginPath<<R as IntoRangePath>::Result>;
    fn into_range_path(self) -> Self::Result {
        OriginPath {
            postfix: self.postfix.into_range_path(),
            origin: self.origin,
        }
    }
}
//impl<P: RangePath> Into<FoundPath> for OriginPath<P> {
//    fn into(self) -> FoundPath {
//        self.postfix.into()
//    }
//}
impl<P: RangePath> RangePath for OriginPath<P> {
    fn into_complete(self) -> Option<Child> {
        self.postfix.into_complete()
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
impl<P: GetCacheKey> GetCacheKey for OriginPath<P> {
    fn cache_key(&self) -> CacheKey {
        self.postfix.cache_key()
    }
}