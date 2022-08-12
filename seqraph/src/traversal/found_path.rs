use super::*;

/// used to represent results after traversal with any path
#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub(crate) enum FoundPath {
    Complete(Child),
    Range(SearchPath),
}
impl From<MatchEnd> for FoundPath {
    fn from(match_end: MatchEnd) -> Self {
        match match_end {
            MatchEnd::Complete(c) => FoundPath::Complete(c),
            MatchEnd::Path(path) => FoundPath::Range(SearchPath::from(path)),
        }
    }
}
impl<
    'a: 'g,
    'g,
> FoundPath {
    pub(crate) fn new<
        T: Tokenize,
        D: MatchDirection,
        Trav: Traversable<'a, 'g, T>,
    >(trav: &'a Trav, path: SearchPath) -> Self {
        if path.is_complete::<_, D, _>(trav) {
            Self::Complete(path.start_match_path().entry().parent)
        } else {
            Self::Range(path)
        }
    }
    pub fn root(&self) -> Child {
        match self {
            Self::Range(path) => path.root(),
            Self::Complete(c) => *c,
        }
    }
    #[track_caller]
    pub fn unwrap_complete(self) -> Child {
        match self {
            Self::Complete(index) => index,
            _ => panic!("Unable to unwrap {:?} as complete.", self),
        }
    }
    #[track_caller]
    #[allow(unused)]
    pub fn unwrap_range(self) -> SearchPath {
        match self {
            Self::Range(path) => path,
            _ => panic!("Unable to unwrap {:?} as range.", self),
        }
    }
    #[track_caller]
    pub fn get_range(&self) -> Option<&SearchPath> {
        match self {
            Self::Range(path) => Some(path),
            _ => None,
        }
    }
    #[track_caller]
    pub fn expect_complete(self, msg: &str) -> Child {
        match self {
            Self::Complete(index) => index,
            _ => panic!("Unable to unwrap {:?} as complete: {}", self, msg),
        }
    }
    fn is_complete(&self) -> bool {
        matches!(self, FoundPath::Complete(_))
    }
}
impl PartialOrd for FoundPath {
    fn partial_cmp(&self, other: &FoundPath) -> Option<Ordering> {
        match (self, other) {
            (FoundPath::Complete(l), FoundPath::Complete(r)) => l.width().partial_cmp(&r.width()),
            (FoundPath::Range(l), FoundPath::Range(r)) =>
                l.partial_cmp(&r),
            _ => self.is_complete().partial_cmp(&self.is_complete())
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
            Self::Range(r) => r.width(),
        }
    }
}