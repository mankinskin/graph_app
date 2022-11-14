use super::*;
use std::hash::Hash;

pub(crate) trait TraversalQuery:
    Advance
    + Debug
    + Clone
    + Hash
    + PartialEq
    + Eq
    + Send
    + Sync
    + 'static
    + Unpin
{}
impl<
    T: Advance
        + Debug
        + Clone
        + Hash
        + PartialEq
        + Eq
        + Send
        + Sync
        + 'static
        + Unpin
> TraversalQuery for T {}

pub(crate) trait TraversalStartPath:
    PathAppend<Result=StartPath>
    //+ BorderPath
    + Clone
    + Debug
{
}
impl<
    T: PathAppend<Result=StartPath>
        //+ BorderPath
        + Clone
        + Debug
> TraversalStartPath for T {}

#[derive(Clone, Debug, PartialEq, Hash, Eq)]
pub(crate) enum PathPair<
    P: NewAdvanced,
    Q: TraversalQuery,
> {
    GraphMajor(P, Q),
    QueryMajor(Q, P),
}
impl<
    P: NewAdvanced,
    Q: TraversalQuery,
> PathPair<P, Q> {
    pub fn from_mode(path: P, query: Q, mode: bool) -> Self {
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
    pub fn unpack(self) -> (P, Q) {
        match self {
            Self::GraphMajor(path, query) =>
                (path, query),
            Self::QueryMajor(query, path) =>
                (path, query),
        }
    }
    #[allow(unused)]
    pub fn get_path(&self) -> &P {
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