use crate::*;

pub trait RangePath:
    IntoRangePath
    + BasePath
    + Into<FoundPath>
    + Hash
    // + PathComplete
{
    fn into_complete(self) -> Option<Child>;

    #[track_caller]
    fn unwrap_complete(self) -> Child {
        self.clone().into_complete()
            .expect(&format!("Unable to unwrap {:?} as complete.", self))
    }
    #[track_caller]
    fn expect_complete(self, msg: &str) -> Child {
        self.clone().into_complete()
            .expect(&format!("Unable to unwrap {:?} as complete: {}", self, msg))
    }
}
impl RangePath for FoundPath {
    fn into_complete(self) -> Option<Child> {
        match self {
            Self::Complete(index) => Some(index),
            _ => None,
        }
    }
}
//impl RangePath for ChildPath<Start> {
//    fn into_complete(self) -> Option<Child> {
//        self.path.is_empty().then(||
//            self.child
//        )
//    }
//}
//impl RangePath for ChildPath<End> {
//    fn into_complete(self) -> Option<Child> {
//        self.path.is_empty().then(||
//            self.child
//        )
//    }
//}

pub trait IntoRangePath {
    type Result: RangePath;
    fn into_range_path(self) -> Self::Result;
}
impl IntoRangePath for FoundPath {
    type Result = Self;
    fn into_range_path(self) -> Self::Result {
        self
    }
}
//impl<R> IntoRangePath for ChildPath<R> {
//    type Result = FoundPath;
//    fn into_range_path(self) -> Self::Result {
//        FoundPath::from(self)
//    }
//}
//impl IntoRangePath for PathLeaf {
//    type Result = FoundPath;
//    fn into_range_path(self) -> Self::Result {
//        FoundPath::from(ChildPath::from(self))
//    }
//}
//impl IntoRangePath for SearchPath {
//    type Result = FoundPath;
//    fn into_range_path(self) -> Self::Result {
//        self
//    }
//}

/// used to represent results after traversal with any path
#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub enum FoundPath {
    Complete(Child),
    Range(SearchPath),
    Postfix(ChildPath<Start>),
    Prefix(ChildPath<End>),
}
impl From<ChildPath<Start>> for FoundPath {
    fn from(path: ChildPath<Start>) -> Self {
        FoundPath::Postfix(path)
    }
}
impl From<ChildPath<End>> for FoundPath {
    fn from(path: ChildPath<End>) -> Self {
        FoundPath::Prefix(path)
    }
}
impl<P: Into<FoundPath>> From<OriginPath<P>> for FoundPath {
    fn from(path: OriginPath<P>) -> Self {
        path.postfix.into()
    }
}
impl<P: MatchEndPath> From<MatchEnd<P>> for FoundPath {
    fn from(match_end: MatchEnd<P>) -> Self {
        match match_end {
            MatchEnd::Path(path) => path.into(),
            MatchEnd::Complete(c) => FoundPath::Complete(c),
        }
    }
}
impl PartialOrd for FoundPath {
    fn partial_cmp(&self, other: &FoundPath) -> Option<Ordering> {
        match (self, other) {
            (FoundPath::Complete(l), FoundPath::Complete(r)) =>
                l.width().partial_cmp(&r.width()),
            // complete always greater than prefix/postfix/range
            (FoundPath::Complete(_), _) => Some(Ordering::Greater),
            (_, FoundPath::Complete(_)) => Some(Ordering::Less),
            (FoundPath::Range(l), FoundPath::Range(r)) =>
                l.partial_cmp(&r),
            // TODO: possibly prefer smaller sub_index
            _ => match self.width().cmp(&other.width()) {
                Ordering::Equal =>
                    self.num_path_segments().partial_cmp(
                        &other.num_path_segments()
                    ).map(Ordering::reverse),
                o => Some(o)
            },
        }
    }
}
impl Ord for FoundPath {
    fn cmp(&self, other: &FoundPath) -> Ordering {
        self.partial_cmp(&other)
            .unwrap_or(Ordering::Equal)
    }
}
impl Wide for FoundPath {
    fn width(&self) -> usize {
        match self {
            Self::Complete(c) => c.width,
            Self::Range(p) => p.width(),
            Self::Prefix(p) => p.width(),
            Self::Postfix(p) => p.width(),
        }
    }
}
impl<
    'a: 'g,
    'g,
> FoundPath {
    fn num_path_segments(&self) -> usize {
        match self {
            Self::Complete(_) => 0,
            Self::Range(p) => HasMatchPaths::num_path_segments(p),
            Self::Prefix(p) => p.num_path_segments(),
            Self::Postfix(p) => p.num_path_segments(),
        }
    }
    //#[allow(unused)]
    //#[track_caller]
    //pub fn unwrap_range(self) -> SearchPath {
    //    match self {
    //        Self::Range(path) => path,
    //        _ => panic!("Unable to unwrap {:?} as range.", self),
    //    }
    //}
    //#[allow(unused)]
    //#[track_caller]
    //pub fn get_range(&self) -> Option<&SearchPath> {
    //    match self {
    //        Self::Range(path) => Some(path),
    //        _ => None,
    //    }
    //}
    //#[allow(unused)]
    //fn is_complete(&self) -> bool {
    //    matches!(self, FoundPath::Complete(_))
    //}
}