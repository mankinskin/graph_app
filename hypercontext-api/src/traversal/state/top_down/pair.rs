use crate::{
    graph::vertex::location::child::ChildLocation,
    impl_cursor_pos,
    path::{
        mutators::append::PathAppend,
        structs::rooted::index_range::IndexRangePath,
    },
    traversal::{
        cache::key::props::LeafKey,
        state::cursor::RangeCursor,
    },
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PathPair {
    pub path: IndexRangePath,
    pub cursor: RangeCursor,
    pub mode: PathPairMode,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PathPairMode {
    GraphMajor,
    QueryMajor,
}

impl_cursor_pos! {
    CursorPosition for PathPair, self => self.cursor.relative_pos
}
impl PathPair {
    pub fn new(
        path: IndexRangePath,
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
impl LeafKey for PathPair {
    fn leaf_location(&self) -> ChildLocation {
        self.path.leaf_location()
    }
}
