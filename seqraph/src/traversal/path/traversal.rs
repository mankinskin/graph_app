use super::*;
use std::hash::Hash;

pub(crate) trait TraversalQuery:
    AdvanceablePath
    + PathFinished
    + Debug
    + Clone
    + Hash
    + PartialEq
    + Eq
{}
impl<
    T: AdvanceablePath
        + ReduciblePath
        + PathFinished
        + Debug
        + Clone
        + Hash
        + PartialEq
        + Eq
> TraversalQuery for T {}

pub(crate) trait TraversalPath:
    AdvanceablePath
    + ReduciblePath
    + GraphStart
    + GraphEnd
    + PathComplete
    + HasMatchPaths
    + Wide
    + Debug
{
    fn reduce_end<
        'a: 'g,
        'g,
        T: Tokenize,
        D: MatchDirection,
        Trav: Traversable<'a, 'g, T>,
    >(self, trav: &'a Trav) -> FoundPath<Self>;
    fn move_width_into_start(&mut self);
    fn on_match<
        'a: 'g,
        'g,
        T: Tokenize,
        D: MatchDirection,
        Trav: Traversable<'a, 'g, T>,
    >(&mut self, trav: &'a Trav);
}

pub(crate) trait TraversalStartPath:
    BorderPath
    + PathAppend<Result=StartPath>
    + Clone
    + Debug
{
}
impl<
    T: BorderPath
        + PathAppend<Result=StartPath>
        + Clone
        + Debug
> TraversalStartPath for T {}
#[derive(Clone, Debug, PartialEq, Hash, Eq)]
pub(crate) enum PathPair<
    Q: TraversalQuery,
    G: TraversalPath,
> {
    GraphMajor(G, Q),
    QueryMajor(Q, G),
}
impl<
    Q: TraversalQuery,
    G: TraversalPath,
> PathPair<Q, G> {
    pub fn from_mode(path: G, query: Q, mode: bool) -> Self {
        if mode {
            Self::GraphMajor(path, query)
        } else {
            Self::QueryMajor(query, path)
        }
    }
    pub fn mode(&self) -> bool {
        matches!(self, Self::GraphMajor(_, _))
    }
    pub fn push_major(&mut self, location: ChildLocation) {
        match self {
            Self::GraphMajor(path, _) =>
                path.push_end(location),
            Self::QueryMajor(query, _) =>
                query.push_end(location),
        }
    }
    pub fn unpack(self) -> (G, Q) {
        match self {
            Self::GraphMajor(path, query) =>
                (path, query),
            Self::QueryMajor(query, path) =>
                (path, query),
        }
    }
}
impl<
    Q: index::IndexingQuery,
    G: TraversalPath,
> PathPair<Q, G> {
    pub(crate) fn reduce_mismatch<
        'a: 'g,
        'g,
        T: Tokenize,
        D: MatchDirection,
        Trav: Traversable<'a, 'g, T>,
    >(self, trav: &'a Trav) -> TraversalResult<G, Q> {
        match self {
            Self::GraphMajor(path, query) |
            Self::QueryMajor(query, path) => {
                TraversalResult::new(
                    FoundPath::new::<_, D, _>(trav, path.reduce_mismatch::<_, D, _>(trav)),
                    query.reduce_mismatch::<_, D, _>(trav),
                )
            }
        }
    }
}