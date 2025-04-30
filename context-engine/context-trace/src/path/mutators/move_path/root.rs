use crate::{
    direction::{
        Direction,
        pattern::PatternDirection,
    },
    path::{
        accessors::{
            child::{
                RootChildIndex,
                RootChildIndexMut,
            },
            role::{
                End,
                PathRole,
            },
            root::RootPattern,
        },
        structs::rooted::{
            role_path::RootedRolePath,
            root::PathRoot,
        },
    },
    trace::has_graph::HasGraph,
};
use std::ops::ControlFlow;

pub trait MoveRootIndex<D: Direction, R: PathRole = End> {
    fn move_root_index<G: HasGraph>(
        &mut self,
        trav: &G,
    ) -> ControlFlow<()>;
}
impl<Root: PathRoot, Role: PathRole, D: PatternDirection> MoveRootIndex<D, Role>
    for RootedRolePath<Role, Root>
{
    fn move_root_index<G: HasGraph>(
        &mut self,
        trav: &G,
    ) -> ControlFlow<()> {
        let graph = trav.graph();
        let pattern = self.root_pattern::<G>(&graph);
        if let Some(next) = D::pattern_index_next(
            pattern,
            RootChildIndex::<Role>::root_child_index(self),
        ) {
            assert!(next < pattern.len());
            *self.root_child_index_mut() = next;
            ControlFlow::Continue(())
        } else {
            ControlFlow::Break(())
        }
    }
}
