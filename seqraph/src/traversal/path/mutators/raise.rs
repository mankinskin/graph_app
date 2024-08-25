use std::borrow::Borrow;

use crate::{
    graph::direction::r#match::MatchDirection,
    traversal::{
        cache::state::parent::ParentState,
        path::mutators::move_path::key::AdvanceKey,
        traversable::{
            TravDir,
            Traversable,
        },
    },
};
use crate::graph::vertex::{
    location::{
        child::ChildLocation,
        pattern::IntoPatternLocation,
    },
    pattern::pattern_width,
};

pub trait PathRaise {
    fn path_raise<Trav: Traversable>(
        &mut self,
        trav: &Trav,
        parent_entry: ChildLocation,
    );
}

impl PathRaise for ParentState {
    fn path_raise<Trav: Traversable>(
        &mut self,
        trav: &Trav,
        parent_entry: ChildLocation,
    ) {
        let path = &mut self.path.role_path.sub_path;
        let root = self.path.root.location.to_child_location(path.root_entry);
        path.root_entry = parent_entry.sub_index;
        self.path.root.location = parent_entry.into_pattern_location();
        let graph = trav.graph();
        let pattern = graph.expect_pattern_at(&root);
        self.prev_pos = self.root_pos;
        self.root_pos
            .advance_key(pattern_width(&pattern[root.sub_index + 1..]));
        if !path.is_empty()
            || TravDir::<Trav>::pattern_index_prev(pattern.borrow(), root.sub_index).is_some()
        {
            path.path.push(root);
        }
    }
}
