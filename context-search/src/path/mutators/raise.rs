use crate::{
    graph::vertex::location::child::ChildLocation,
    traversal::traversable::Traversable,
};

pub trait PathRaise {
    fn path_raise<Trav: Traversable>(
        &mut self,
        trav: &Trav,
        parent_entry: ChildLocation,
    );
}
