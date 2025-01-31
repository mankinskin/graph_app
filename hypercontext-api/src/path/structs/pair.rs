use crate::{
    graph::vertex::location::child::ChildLocation,
    path::mutators::append::PathAppend,
    traversal::state::cursor::RangeCursor,
};

use super::rooted::index_range::SearchPath;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PathPair {
    pub path: SearchPath,
    pub cursor: RangeCursor,
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
        cursor: RangeCursor,
        mode: PathPairMode,
    ) -> Self {
        Self { path, cursor, mode }
    }
    pub fn push_major(
        &mut self,
        location: ChildLocation,
    ) {
        match self.mode {
            PathPairMode::GraphMajor => self.path.path_append(location),
            PathPairMode::QueryMajor => self.cursor.path_append(location),
        }
    }
}
