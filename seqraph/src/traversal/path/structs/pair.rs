use crate::*;

#[derive(Clone, Debug, PartialEq, Hash, Eq)]
pub enum PathPair<
    P: Advanced,
    Q: QueryPath,
> {
    GraphMajor(P, Q),
    QueryMajor(Q, P),
}
#[derive(Clone, Debug, PartialEq, Hash, Eq)]
pub enum PathPairMode {
    GraphMajor,
    QueryMajor,
}
impl<
    P: Advanced,
    Q: QueryPath,
> PathPair<P, Q> {
    pub fn from_mode(path: P, query: Q, mode: PathPairMode) -> Self {
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
    pub fn push_major(&mut self, location: ChildLocation) {
        match self {
            Self::GraphMajor(path, _) =>
                path.path_append(location),
            Self::QueryMajor(query, _) =>
                query.path_append(location),
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