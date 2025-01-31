use crate::{
    graph::vertex::location::child::ChildLocation,
    path::structs::pair::PathPair,
    traversal::state::{
        child::ChildState,
        end::RangeEnd,
    },
};

pub trait LeafKey {
    fn leaf_location(&self) -> ChildLocation;
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
