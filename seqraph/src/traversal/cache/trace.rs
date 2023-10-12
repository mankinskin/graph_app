use crate::*;

pub trait Trace {
    fn trace<Trav: Traversable>(&self, trav: &Trav, cache: &mut TraversalCache);
}
impl Trace for EndState {
    fn trace<Trav: Traversable>(
        &self,
        trav: &Trav,
        cache: &mut TraversalCache,
    ) {
        match &self.kind {
            EndKind::Range(p) => {
                let root_entry = p.path
                    .role_root_child_location::<Start>()
                    .sub_index;
                cache.trace_path(
                    trav,
                    root_entry,
                    &p.path,
                    self.root_pos,
                    true,
                )
            },
            EndKind::Prefix(p) =>
                cache.trace_path(
                    trav,
                    0,
                    &p.path,
                    self.root_pos,
                    true,
                ),
            _ => {}
        }
    }
}
impl Trace for ChildState {
    fn trace<Trav: Traversable>(&self, trav: &Trav, cache: &mut TraversalCache) {
        cache.trace_path(
            trav,
            self.paths.path.role_root_child_location::<Start>().sub_index,
            &self.paths.path,
            self.root_pos,
            false,
        );
    }
}