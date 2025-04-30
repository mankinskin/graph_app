use crate::{
    direction::Direction,
    graph::vertex::location::child::ChildLocation,
    path::{
        accessors::role::PathRole,
        mutators::{
            append::PathAppend,
            move_path::root::MoveRootIndex,
            pop::PathPop,
        },
    },
    trace::has_graph::HasGraph,
};
use std::ops::ControlFlow;

pub trait MovePath<D: Direction, R: PathRole>:
    PathPop + PathAppend + MoveRootIndex<D, R>
{
    fn move_path_segment<G: HasGraph>(
        &mut self,
        location: &mut ChildLocation,
        trav: &G::Guard<'_>,
    ) -> ControlFlow<()>;

    fn move_path<G: HasGraph>(
        &mut self,
        trav: &G,
    ) -> ControlFlow<()> {
        let graph = trav.graph();
        if let Some(location) = std::iter::from_fn(|| {
            self.path_pop().map(|mut location| {
                self.move_path_segment::<G>(&mut location, &graph)
                    .is_continue()
                    .then_some(location)
            })
        })
        .find_map(|location| location)
        {
            self.path_append(location);
            ControlFlow::Continue(())
        } else {
            self.move_root_index(trav)
        }
    }
}
