use std::fmt::Debug;

use accessors::{
    child::{
        root::GraphRootChild,
        LeafChild,
    },
    has_path::HasRolePath,
    role::PathRole,
};
use structs::role_path::RolePath;

use crate::{
    graph::vertex::{
        child::Child,
        location::child::ChildLocation,
    },
    path::accessors::child::RootChildPos,
    traversal::traversable::Traversable,
};

pub mod accessors;
pub mod mutators;
pub mod structs;

pub trait BaseQuery: Debug + Clone + PartialEq + Eq + Send + Sync + 'static {}

impl<T: Debug + Clone + PartialEq + Eq + Send + Sync + 'static> BaseQuery for T {}

pub trait BasePath: Debug + Sized + Clone + PartialEq + Eq + Send + Sync + Unpin + 'static {}

impl<T: Debug + Sized + Clone + PartialEq + Eq + Send + Sync + Unpin + 'static> BasePath for T {}

pub trait RoleChildPath {
    fn role_leaf_child_location<R: PathRole>(&self) -> Option<ChildLocation>
    where
        Self: LeafChild<R>,
    {
        LeafChild::<R>::leaf_child_location(self)
    }
    fn role_root_child_pos<R: PathRole>(&self) -> usize
    where
        Self: GraphRootChild<R>,
    {
        GraphRootChild::<R>::root_child_location(self).sub_index
    }
    fn role_root_child_location<R: PathRole>(&self) -> ChildLocation
    where
        Self: GraphRootChild<R>,
    {
        GraphRootChild::<R>::root_child_location(self)
    }
    fn child_path_mut<R: PathRole>(&mut self) -> &mut RolePath<R>
    where
        Self: HasRolePath<R>,
    {
        HasRolePath::<R>::role_path_mut(self)
    }
    fn role_leaf_child<R: PathRole, Trav: Traversable>(
        &self,
        trav: &Trav,
    ) -> Child
    where
        Self: LeafChild<R>,
    {
        LeafChild::<R>::leaf_child(self, trav)
    }
    fn child_pos<R: PathRole>(&self) -> usize
    where
        Self: HasRolePath<R>,
    {
        HasRolePath::<R>::role_path(self).root_child_pos()
    }
    fn raw_child_path<R: PathRole>(&self) -> &Vec<ChildLocation>
    where
        Self: HasRolePath<R>,
    {
        HasRolePath::<R>::role_path(self).path()
    }
    fn raw_child_path_mut<R: PathRole>(&mut self) -> &mut Vec<ChildLocation>
    where
        Self: HasRolePath<R>,
    {
        HasRolePath::<R>::role_path_mut(self).path_mut()
    }
}

impl<T> RoleChildPath for T {}
