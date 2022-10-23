use crate::*;

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub(crate) struct OriginPath<P> {
    pub(crate) postfix: P,
    pub(crate) origin: MatchEnd<StartPath>,
}

impl<P: Into<StartPath>> From<P> for OriginPath<StartPath> {
    fn from(start: P) -> Self {
        let origin = start.into();
        OriginPath {
            postfix: origin.clone(),
            origin: MatchEnd::Path(origin),
        }
    }
}
//impl<Q: Into<StartPath>> From<OriginPath<Q>> for StartPath {
//    fn from(start: OriginPath<Q>) -> Self {
//        start.postfix.into()
//    }
//}
impl From<OriginPath<SearchPath>> for OriginPath<StartPath> {
    fn from(other: OriginPath<SearchPath>) -> Self {
        OriginPath {
            postfix: StartPath::from(other.postfix),
            origin: other.origin,
        }
    }
}
impl From<OriginPath<StartPath>> for OriginPath<MatchEnd<StartPath>> {
    fn from(other: OriginPath<StartPath>) -> Self {
        OriginPath {
            postfix: MatchEnd::from(other.postfix),
            origin: other.origin,
        }
    }
}
impl From<OriginPath<MatchEnd<StartPath>>> for OriginPath<FoundPath> {
    fn from(other: OriginPath<MatchEnd<StartPath>>) -> Self {
        OriginPath {
            postfix: FoundPath::from(other.postfix),
            origin: other.origin,
        }
    }
}
impl From<OriginPath<StartPath>> for OriginPath<SearchPath> {
    fn from(other: OriginPath<StartPath>) -> Self {
        OriginPath {
            postfix: SearchPath::from(other.postfix),
            origin: other.origin,
        }
    }
}
impl From<OriginPath<StartLeaf>> for OriginPath<StartPath> {
    fn from(other: OriginPath<StartLeaf>) -> Self {
        OriginPath {
            postfix: StartPath::from(other.postfix),
            origin: other.origin,
        }
    }
}
pub(crate) trait Origin {
    fn into_origin(self) -> MatchEnd<StartPath>;
}
impl<P> Origin for OriginPath<P> {
    fn into_origin(self) -> MatchEnd<StartPath> {
        self.origin
    }
}
impl<P: GraphEntry> GraphEntry for OriginPath<P> {
    fn entry(&self) -> ChildLocation {
        self.postfix.entry()
    }
}
impl<P: RootChild> RootChild for OriginPath<P> {
    fn root_child(&self) -> Child {
        self.postfix.root_child()
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
impl<A: Advanced, F: FromAdvanced<A>> FromAdvanced<A> for OriginPath<F> {
    fn from_advanced<
        'a: 'g,
        'g,
        T: Tokenize,
        D: MatchDirection,
        Trav: Traversable<'a, 'g, T>
    >(path: A, trav: &'a Trav) -> Self {
        Self {
            origin: MatchEnd::Path(path.start_match_path().clone()),
            postfix: F::from_advanced::<_, D, _>(path, trav),
        }
    }
}
//impl<P: RangePath> PathComplete for OriginPath<P> {
//    fn complete<
//        'a: 'g,
//        'g,
//        T: Tokenize,
//        D: MatchDirection,
//        Trav: Traversable<'a, 'g, T>,
//    >(&self, trav: &'a Trav) -> Option<Child> {
//        self.postfix.complete::<_, D, _>(trav)
//    }
//}
impl<P: PathReduce> PathReduce for OriginPath<P> {
    fn into_reduced<
        'a: 'g,
        'g,
        T: Tokenize,
        D: MatchDirection,
        Trav: Traversable<'a, 'g, T>,
    >(mut self, trav: &'a Trav) -> Self {
        self.postfix.reduce::<_, D, _>(trav);
        self
    }
}
impl<P: PathAppend> PathAppend for OriginPath<P>
    where <P as PathAppend>::Result: PathAppend<Result=<P as PathAppend>::Result> + RangePath + GraphEntry
{
    type Result = OriginPath<<P as PathAppend>::Result>;
    fn append<
        'a: 'g,
        'g,
        T: Tokenize,
        D: MatchDirection,
        Trav: Traversable<'a, 'g, T>
    >(self, trav: &'a Trav, parent_entry: ChildLocation) -> Self::Result {
        OriginPath {
            origin: MatchEnd::Path(self.origin.append::<_, D, _>(trav, parent_entry)),
            postfix: self.postfix.append::<_, D, _>(trav, parent_entry),
        }
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