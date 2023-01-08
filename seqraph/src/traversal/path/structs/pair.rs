use crate::*;

#[derive(Clone, Debug, PartialEq, Hash, Eq)]
pub enum PathPair<
    P: Advanced,
    Q: BaseQuery,
> {
    GraphMajor(P, Q),
    QueryMajor(Q, P),
}
impl<
    P: Advanced,
    Q: BaseQuery,
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
                HasPath::<End>::path_mut(path).push(location),
            Self::QueryMajor(query, _) =>
                query.path_mut().push(location),
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