use crate::traversal::{
    cache::state::{
        child::ChildState,
        end::RangeEnd,
    },
    path::structs::{
        pair::PathPair,
        rooted_path::SearchPath,
    },
};
use crate::graph::vertex::location::child::ChildLocation;

pub trait LeafKey {
    fn leaf_location(&self) -> ChildLocation;
}

impl LeafKey for SearchPath {
    fn leaf_location(&self) -> ChildLocation {
        self.end.path.last().cloned().unwrap_or(
            self.root
                .location
                .to_child_location(self.end.sub_path.root_entry),
        )
    }
}

impl LeafKey for ChildState {
    fn leaf_location(&self) -> ChildLocation {
        self.paths.leaf_location()
    }
}

impl LeafKey for PathPair {
    fn leaf_location(&self) -> ChildLocation {
        self.path.leaf_location()
    }
}

impl LeafKey for RangeEnd {
    fn leaf_location(&self) -> ChildLocation {
        self.path.leaf_location()
    }
}
