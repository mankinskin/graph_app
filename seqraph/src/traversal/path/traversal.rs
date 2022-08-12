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
        + Debug
        + Clone
        + Hash
        + PartialEq
        + Eq
> TraversalQuery for T {}

//pub(crate) trait ReduciblePath:
//    AdvanceablePath
//    + EndPathMut
//    + GraphStart
//    + GraphEnd
//    + PathComplete
//    + HasMatchPaths
//    + ExitMut
//    + End
//    + EntryPos
//    + WideMut
//    + Wide
//    + Debug
//    + PartialOrd
//    + Clone
//{
//}
//pub(crate) trait TraversalPath:
//    AdvanceablePath
//    + EndPathMut
//    + GraphStart
//    + GraphEnd
//    + PathComplete
//    + HasMatchPaths
//    + ExitMut
//    + End
//    + EntryPos
//    + WideMut
//    + Wide
//    + Debug
//    + PartialOrd
//    + Clone
//{
//}

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
> {
    GraphMajor(SearchPath, Q),
    QueryMajor(Q, SearchPath),
}
impl<
    Q: TraversalQuery,
> PathPair<Q> {
    pub fn from_mode(path: SearchPath, query: Q, mode: bool) -> Self {
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
    pub fn unpack(self) -> (SearchPath, Q) {
        match self {
            Self::GraphMajor(path, query) =>
                (path, query),
            Self::QueryMajor(query, path) =>
                (path, query),
        }
    }
    #[allow(unused)]
    pub fn get_path(&self) -> &SearchPath {
        match self {
            Self::GraphMajor(path, _) |
            Self::QueryMajor(_, path) =>
                path,
        }
    }
    #[allow(unused)]
    pub fn get_query(&self) -> &Q {
        match self {
            Self::GraphMajor(_, query) |
            Self::QueryMajor(query, _) =>
                query,
        }
    }
}
//impl<
//    Q: index::IndexingQuery,
//    G: TraversalPath,
//> PathPair<Q, G> {
//    pub(crate) fn reduce_mismatch<
//        'a: 'g,
//        'g,
//        T: Tokenize,
//        D: MatchDirection,
//        Trav: Traversable<'a, 'g, T>,
//    >(self, trav: &'a Trav) -> TraversalResult<G, Q> {
//        match self {
//            Self::GraphMajor(path, query) |
//            Self::QueryMajor(query, path) => {
//                TraversalResult::new(
//                    FoundPath::new::<_, D, _>(trav, path.reduce_mismatch::<_, D, _>(trav)),
//                    query,
//                )
//            }
//        }
//    }
//}