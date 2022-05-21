pub(crate) mod bft;
pub(crate) mod dft;
pub(crate) mod path;
pub(crate) mod node;
pub(crate) mod traversable;
pub(crate) mod folder;
pub(crate) mod iterator;
pub(crate) mod policy;

use std::cmp::Ordering;

pub(crate) use super::*;
pub(crate) use bft::*;
pub(crate) use dft::*;
pub(crate) use path::*;
pub(crate) use node::*;
pub(crate) use traversable::*;
pub(crate) use folder::*;
pub(crate) use iterator::*;
pub(crate) use policy::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TraversalResult<P: TraversalPath, Q: TraversalQuery> {
    pub(crate) found: FoundPath<P>,
    pub(crate) query: Q,
}

impl<P: TraversalPath, Q: TraversalQuery> TraversalResult<P, Q> {
    pub(crate) fn new(found: FoundPath<P>, query: Q) -> Self {
        Self {
            found,
            query,
        }
    }
    #[track_caller]
    pub fn unwrap_complete(self) -> Child {
        self.found.unwrap_complete()
    }
    #[track_caller]
    pub fn expect_complete(self, msg: &str) -> Child {
        self.found.expect_complete(msg)
    }
}
impl<P: TraversalPath, Q: QueryPath> TraversalResult<P, Q> {
    pub fn complete(query: impl IntoPattern, index: impl AsChild) -> Self {
        Self {
            found: FoundPath::Complete(index.as_child()),
            query: Q::complete(query),
        }
    }
}

#[derive(Debug, Clone, Eq)]
pub(crate) enum FoundPath<P: TraversalPath> {
    Complete(Child),
    Range(P),
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
            Self::Complete(path.get_start_match_path().entry().parent)
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
}
impl<P: TraversalPath> ResultOrd for FoundPath<P> {
    fn is_complete(&self) -> bool {
        matches!(self, FoundPath::Complete(_))
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
impl<Rhs: ResultOrd, P: TraversalPath> PartialOrd<Rhs> for FoundPath<P> {
    fn partial_cmp(&self, other: &Rhs) -> Option<Ordering> {
        Some(ResultOrd::cmp(self, other))
    }
}
impl<Rhs: ResultOrd, P: TraversalPath> PartialEq<Rhs> for FoundPath<P> {
    fn eq(&self, other: &Rhs) -> bool {
        ResultOrd::eq(self, other)
    }
}