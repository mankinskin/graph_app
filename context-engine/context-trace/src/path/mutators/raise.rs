use crate::{
    graph::vertex::location::child::ChildLocation,
    trace::has_graph::HasGraph,
};

pub trait PathRaise {
    fn path_raise<G: HasGraph>(
        &mut self,
        trav: &G,
        parent_entry: ChildLocation,
    );
}
