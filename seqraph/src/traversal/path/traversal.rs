use super::*;
use std::hash::Hash;

pub(crate) trait TraversalQuery:
    AdvanceablePath
    + Debug
    + Clone
    + Hash
    + PartialEq
    + Eq
{}
impl<
    T: AdvanceablePath
        + ReduciblePath
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
    + WideMut
    + Debug
    + PartialOrd
{
    fn reduce_end<
        'a: 'g,
        'g,
        T: Tokenize,
        D: MatchDirection,
        Trav: Traversable<'a, 'g, T>,
    >(mut self, trav: &'a Trav) -> FoundPath<Self> {
        let graph = trav.graph();
        // remove segments pointing to mismatch at pattern head
        while let Some(location) = self.end_path_mut().pop() {
            let pattern = graph.expect_pattern_at(&location);
            // skip segments at end of pattern
            if D::pattern_index_next(pattern.borrow(), location.sub_index).is_some() {
                self.end_path_mut().push(location);
                break;
            }
        }
        FoundPath::new::<_, D, _>(trav, self)
    }

    fn add_match_width<
        'a: 'g,
        'g,
        T: Tokenize,
        D: MatchDirection,
        Trav: Traversable<'a, 'g, T>,
    >(&mut self, trav: &'a Trav) {
        if let Some(end) = self.get_end::<_, D, _>(trav) {
            let wmut = self.width_mut();
            *wmut += end.width;
        }
    }
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
                    query,
                )
            }
        }
    }
}