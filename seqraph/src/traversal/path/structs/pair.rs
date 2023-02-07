use crate::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PathPair {
    GraphMajor(SearchPath, QueryState),
    QueryMajor(QueryState, SearchPath),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PathPairMode {
    GraphMajor,
    QueryMajor,
}
impl<
> PathPair {
    pub fn from_mode(path: SearchPath, query: QueryState, mode: PathPairMode) -> Self {
        if matches!(mode, PathPairMode::GraphMajor) {
            Self::GraphMajor(path, query)
        } else {
            Self::QueryMajor(query, path)
        }
    }
    pub fn mode(&self) -> PathPairMode {
        match self {
            Self::GraphMajor(_, _) => PathPairMode::GraphMajor,
            Self::QueryMajor(_, _) => PathPairMode::QueryMajor,
        }
    }
    pub fn push_major
    (
        &mut self,
        location: ChildLocation,
    ) {
        match self {
            Self::GraphMajor(path, _) =>
                path.path_append(location),
            Self::QueryMajor(query, _) =>
                query.path_append(location),
        }
    }
    pub fn unpack(self) -> (SearchPath, QueryState) {
        match self {
            Self::GraphMajor(path, query) =>
                (path, query),
            Self::QueryMajor(query, path) =>
                (path, query),
        }
    }
    pub fn path(&self) -> &SearchPath {
        match self {
            Self::GraphMajor(path, _) |
            Self::QueryMajor(_, path) =>
                path,
        }
    }
    pub fn query(&self) -> &QueryState {
        match self {
            Self::GraphMajor(_, query) |
            Self::QueryMajor(query, _) =>
                query,
        }
    }
}