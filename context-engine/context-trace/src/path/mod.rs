use std::fmt::Debug;

use accessors::{
    child::{
        LeafChild,
        root::GraphRootChild,
    },
    has_path::HasRolePath,
    role::PathRole,
};
use structs::role_path::RolePath;

use crate::{
    direction::pattern::PatternDirection,
    graph::vertex::{
        child::Child,
        location::child::ChildLocation,
    },
    path::accessors::child::RootChildIndex,
    trace::has_graph::HasGraph,
};

pub mod accessors;
pub mod mutators;
pub mod structs;

pub trait BaseQuery:
    Debug + Clone + PartialEq + Eq + Send + Sync + 'static
{
}

impl<T: Debug + Clone + PartialEq + Eq + Send + Sync + 'static> BaseQuery
    for T
{
}

pub trait BasePath:
    Debug + Sized + Clone + PartialEq + Eq + Send + Sync + Unpin + 'static
{
}

impl<T: Debug + Sized + Clone + PartialEq + Eq + Send + Sync + Unpin + 'static>
    BasePath for T
{
}

pub trait RolePathUtils {
    fn role_leaf_child_location<R: PathRole>(&self) -> Option<ChildLocation>
    where
        Self: LeafChild<R>,
    {
        LeafChild::<R>::leaf_child_location(self)
    }
    fn role_root_child_index<R: PathRole>(&self) -> usize
    where
        Self: RootChildIndex<R>,
    {
        RootChildIndex::<R>::root_child_index(self)
    }
    fn role_root_child_location<R: PathRole>(&self) -> ChildLocation
    where
        Self: GraphRootChild<R>,
    {
        GraphRootChild::<R>::root_child_location(self)
    }

    fn role_outer_width<G: HasGraph, R: PathRole>(
        &self,
        trav: &G,
    ) -> usize
    where
        Self: GraphRootChild<R>,
    {
        self.get_outer_width(trav)
    }
    fn role_inner_width<G: HasGraph, R: PathRole>(
        &self,
        trav: &G,
    ) -> usize
    where
        Self: GraphRootChild<R>,
    {
        self.get_inner_width(trav)
    }
    fn is_at_border<G: HasGraph, R: PathRole>(
        &self,
        trav: G,
    ) -> bool
    where
        Self: GraphRootChild<R>,
    {
        let graph = trav.graph();
        let location = self.role_root_child_location::<R>();
        let pattern = graph.expect_pattern_at(location);
        <R::BorderDirection as PatternDirection>::pattern_index_next(
            pattern,
            location.sub_index,
        )
        .is_none()
    }
    fn child_path_mut<R: PathRole>(&mut self) -> &mut RolePath<R>
    where
        Self: HasRolePath<R>,
    {
        HasRolePath::<R>::role_path_mut(self)
    }
    fn role_leaf_child<R: PathRole, G: HasGraph>(
        &self,
        trav: &G,
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
        HasRolePath::<R>::role_path(self).root_child_index()
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

impl<T> RolePathUtils for T {}
