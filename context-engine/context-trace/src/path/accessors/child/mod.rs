use auto_impl::auto_impl;
use root::RootChild;

pub mod root;

use crate::{
    graph::vertex::{
        child::Child,
        location::child::ChildLocation,
    },
    path::accessors::{
        has_path::HasPath,
        role::PathRole,
    },
    trace::traversable::Traversable,
};
pub trait LeafChild<R>: RootChildPos<R> {
    fn leaf_child_location(&self) -> Option<ChildLocation>;
    fn leaf_child<Trav: Traversable>(
        &self,
        trav: &Trav,
    ) -> Child;
}

impl<R: PathRole, P: RootChild<R> + PathChild<R>> LeafChild<R> for P {
    fn leaf_child_location(&self) -> Option<ChildLocation> {
        self.path_child_location()
    }
    fn leaf_child<Trav: Traversable>(
        &self,
        trav: &Trav,
    ) -> Child {
        self.path_child(trav)
            .unwrap_or_else(|| self.root_child(trav))
    }
}

pub trait LeafChildPosMut<R>: RootChildPosMut<R> {
    fn leaf_child_pos_mut(&mut self) -> &mut usize;
}

/// used to get a descendant in a Graph, pointed to by a child path
#[auto_impl(& mut)]
pub trait PathChild<R: PathRole>: HasPath<R> {
    fn path_child_location(&self) -> Option<ChildLocation> {
        R::bottom_up_iter(self.path().iter()).next().cloned() as Option<_>
    }
    fn path_child_location_mut(&mut self) -> Option<&mut ChildLocation> {
        R::bottom_up_iter(self.path_mut().iter_mut()).next()
    }
    fn path_child<Trav: Traversable>(
        &self,
        trav: &Trav,
    ) -> Option<Child> {
        self.path_child_location()
            .map(|loc| trav.graph().expect_child_at(loc).clone())
    }
}

/// access to the position of a child
#[auto_impl(&, & mut)]
pub trait RootChildPos<R> {
    fn root_child_pos(&self) -> usize;
}

pub trait RootChildPosMut<R>: RootChildPos<R> {
    fn root_child_pos_mut(&mut self) -> &mut usize;
}
