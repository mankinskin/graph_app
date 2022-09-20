use super::*;

/// used to represent results after traversal with any path
#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub(crate) enum FoundPath {
    Complete(Child),
    Range(SearchPath),
    Postfix(StartPath),
    Prefix(EndPath),
}
impl From<MatchEnd<StartPath>> for FoundPath {
    fn from(match_end: MatchEnd<StartPath>) -> Self {
        match match_end {
            MatchEnd::Complete(c) => FoundPath::Complete(c),
            MatchEnd::Path(path) => FoundPath::Postfix(path),
        }
    }
}
impl RootChild for FoundPath {
    fn root_child(&self) -> Child {
        match self {
            Self::Range(path) => path.root_child(),
            Self::Postfix(path) => path.root_child(),
            Self::Prefix(path) => path.root_child(),
            Self::Complete(c) => *c,
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
    pub(crate) fn new<
        T: Tokenize,
        D: MatchDirection,
        Trav: Traversable<'a, 'g, T>,
    >(trav: &'a Trav, path: SearchPath) -> Self {
        if path.is_complete::<_, D, _>(trav) {
            Self::Complete(path.start_match_path().get_entry_location().parent)
        } else {
            //path.reduce_end::<_, D, _>(trav);
            Self::Range(path)
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
    pub fn expect_complete(self, msg: &str) -> Child {
        match self {
            Self::Complete(index) => index,
            _ => panic!("Unable to unwrap {:?} as complete: {}", self, msg),
        }
    }
    fn num_path_segments(&self) -> usize {
        match self {
            Self::Complete(_) => 0,
            Self::Range(p) => HasMatchPaths::num_path_segments(p),
            Self::Prefix(p) => p.num_path_segments(),
            Self::Postfix(p) => p.num_path_segments(),
        }
    }
    #[allow(unused)]
    #[track_caller]
    pub fn unwrap_range(self) -> SearchPath {
        match self {
            Self::Range(path) => path,
            _ => panic!("Unable to unwrap {:?} as range.", self),
        }
    }
    #[allow(unused)]
    #[track_caller]
    pub fn get_range(&self) -> Option<&SearchPath> {
        match self {
            Self::Range(path) => Some(path),
            _ => None,
        }
    }
    #[allow(unused)]
    fn is_complete(&self) -> bool {
        matches!(self, FoundPath::Complete(_))
    }
}