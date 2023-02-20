use crate::*;

pub trait PathRaise {
    fn path_raise<
        Trav: Traversable,
    >(
        &mut self,
        trav: &Trav,
        parent_entry: ChildLocation
    );
}
impl PathRaise for ParentState {
    fn path_raise<
        Trav: Traversable,
    >(
        &mut self,
        trav: &Trav,
        parent_entry: ChildLocation,
    ) {
        let mut path = &mut self.path.split_path;
        let root = path.root.location.to_child_location(path.sub_path.root_entry);
        path.sub_path.root_entry = parent_entry.sub_index;
        path.root.location = parent_entry.into_pattern_location();
        let graph = trav.graph();
        let pattern = graph.expect_pattern_at(&root);
        self.prev_pos = self.root_pos;
        self.root_pos.advance_key(pattern[root.sub_index+1..].iter().fold(0, |a, c| a + c.width()));
        if !path.sub_path.is_empty() ||
            TravDir::<Trav>::pattern_index_prev(pattern.borrow(), root.sub_index).is_some()
        {
            path.sub_path.path.push(root);
        }
    }
}