use crate::{
    *,
    ChildLocation,
    Tokenize,
    MatchDirection,
    QueryResult,
    FoundPath,
};
use super::*;

pub trait TraversalQuery:
    AdvanceablePath
    + PatternStart
    + PatternEnd
    + PathFinished
    + Debug
{}
impl<
    T: AdvanceablePath
        + PatternStart
        + PatternEnd
        + PathFinished
        + Debug,
> TraversalQuery for T {}

pub(crate) trait TraversalPath:
    AdvanceablePath
    + GraphStart
    + GraphEnd
    + From<StartPath>
    + Into<StartPath>
    + Into<GraphRangePath>
    + Debug
{
    fn reduce_end<
        'a: 'g,
        'g,
        T: Tokenize + 'a,
        D: MatchDirection + 'a,
        Trav: Traversable<'a, 'g, T>,
    >(self, trav: &'a Trav) -> FoundPath;
    fn move_width_into_start(&mut self);
    fn on_match<
        'a: 'g,
        'g,
        T: Tokenize + 'a,
        D: MatchDirection + 'a,
        Trav: Traversable<'a, 'g, T>,
    >(&mut self, trav: &'a Trav);
}

#[derive(Clone, Debug)]
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
                path.push_next(location),
            Self::QueryMajor(query, _) =>
                query.push_next(location),
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
    pub(crate) fn reduce_mismatch<
        'a: 'g,
        'g,
        T: Tokenize + 'a,
        D: MatchDirection + 'a,
        Trav: Traversable<'a, 'g, T>,
    >(self, trav: &'a Trav) -> QueryResult<Q> {
        match self {
            Self::GraphMajor(path, query) |
            Self::QueryMajor(query, path) => {
                QueryResult::new(
                    FoundPath::new::<_, D, _>(trav, path.reduce_mismatch::<_, D, _>(trav).into()),
                    query.reduce_mismatch::<_, D, _>(trav),
                )
            }
        }
    }
}