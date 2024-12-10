use crate::traversal::{
    cache::state::query::QueryState,
    path::{
        mutators::append::PathAppend,
        structs::rooted_path::SearchPath,
    },
};
use crate::graph::vertex::location::child::ChildLocation;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PathPair {
    pub path: SearchPath,
    pub query: QueryState,
    pub mode: PathPairMode,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PathPairMode {
    GraphMajor,
    QueryMajor,
}

impl PathPair {
    pub fn new(
        path: SearchPath,
        query: QueryState,
        mode: PathPairMode,
    ) -> Self {
        Self { path, query, mode }
    }
    pub fn push_major(
        &mut self,
        location: ChildLocation,
    ) {
        match self.mode {
            PathPairMode::GraphMajor => self.path.path_append(location),
            PathPairMode::QueryMajor => self.query.path_append(location),
        }
    }
}
