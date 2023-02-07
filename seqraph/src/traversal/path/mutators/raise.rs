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
impl PathRaise for RootedRolePath<Start, PatternLocation> {
    fn path_raise<
        Trav: Traversable,
    >(
        &mut self,
        trav: &Trav,
        parent_entry: ChildLocation,
    ) {
        let prev = self.split_path.root.to_child_location(self.split_path.sub_path.root_entry);
        self.split_path.sub_path.root_entry = parent_entry.sub_index;
        self.split_path.root = parent_entry.into_pattern_location();

        if !self.split_path.sub_path.is_empty() || {
            let graph = trav.graph();
            let pattern = graph.expect_pattern_at(&prev);
            TravDir::<Trav>::pattern_index_prev(pattern.borrow(), prev.sub_index).is_some()
        } {
            self.split_path.sub_path.path.push(prev);
        }
    }
}