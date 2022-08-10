pub(crate) mod bft;
pub(crate) mod dft;
pub(crate) mod path;
pub(crate) mod node;
pub(crate) mod traversable;
pub(crate) mod folder;
pub(crate) mod iterator;
pub(crate) mod policy;
pub(crate) mod match_end;

pub(crate) use super::*;
pub(crate) use bft::*;
pub(crate) use dft::*;
pub(crate) use path::*;
pub(crate) use node::*;
pub(crate) use traversable::*;
pub(crate) use folder::*;
pub(crate) use iterator::*;
pub(crate) use policy::*;
pub(crate) use match_end::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct TraversalResult<P: TraversalPath, Q: TraversalQuery> {
    pub(crate) found: FoundPath<P>,
    pub(crate) query: Q,
}

impl<P: TraversalPath, Q: TraversalQuery> TraversalResult<P, Q> {
    pub(crate) fn new(found: impl Into<FoundPath<P>>, query: Q) -> Self {
        Self {
            found: found.into(),
            query,
        }
    }
    #[track_caller]
    pub fn unwrap_complete(self) -> Child {
        self.found.unwrap_complete()
    }
    #[allow(unused)]
    #[track_caller]
    pub fn expect_complete(self, msg: &str) -> Child {
        self.found.expect_complete(msg)
    }
}
impl<P: TraversalPath, Q: QueryPath> TraversalResult<P, Q> {
    #[allow(unused)]
    pub fn complete(query: impl IntoPattern, index: impl AsChild) -> Self {
        Self {
            found: FoundPath::Complete(index.as_child()),
            query: Q::complete(query),
        }
    }
}

#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub(crate) enum FoundPath<P: TraversalPath> {
    Complete(Child),
    Range(P),
}
impl From<MatchEnd> for FoundPath<SearchPath> {
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
    P: TraversalPath,
> FoundPath<P> {
    pub(crate) fn new<
        T: Tokenize,
        D: MatchDirection,
        Trav: Traversable<'a, 'g, T>,
    >(trav: &'a Trav, path: P) -> Self {
        if path.is_complete::<_, D, _>(trav) {
            Self::Complete(path.start_match_path().entry().parent)
        } else {
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
    fn is_complete(&self) -> bool {
        matches!(self, FoundPath::Complete(_))
    }
}
impl<P: TraversalPath + PartialEq> PartialOrd for FoundPath<P> {
    fn partial_cmp(&self, other: &FoundPath<P>) -> Option<Ordering> {
        let l = self.is_complete();
        let r = other.is_complete();
        if l == r {
            self.width().partial_cmp(&other.width())
        } else {
            Some(l.cmp(&r))
        }
    }
}
impl<P: TraversalPath + Eq> Ord for FoundPath<P> {
    fn cmp(&self, other: &FoundPath<P>) -> Ordering {
        self.partial_cmp(&other)
            .unwrap_or(Ordering::Equal)
    }
}
impl<P: TraversalPath> Wide for FoundPath<P> {
    fn width(&self) -> usize {
        match self {
            Self::Complete(c) => c.width,
            Self::Range(r) => r.width(),
        }
    }
}