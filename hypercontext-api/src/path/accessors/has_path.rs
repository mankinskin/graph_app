use crate::{
    graph::vertex::location::child::ChildLocation,
    path::{
        accessors::role::{
            End,
            PathRole,
            Start,
        },
        structs::{
            query_range_path::FoldablePath,
            role_path::RolePath,
            rooted::{
                role_path::RootedRolePath,
                root::RootedPath,
            },
        },
    },
    traversal::state::cursor::PathCursor,
};
use auto_impl::auto_impl;

/// access to a rooted path pointing to a descendant
#[auto_impl(& mut)]
pub trait HasPath<R> {
    fn path(&self) -> &Vec<ChildLocation>;
    fn path_mut(&mut self) -> &mut Vec<ChildLocation>;
}

impl<R: PathRole, P: FoldablePath + HasPath<R>> HasPath<R> for PathCursor<P> {
    fn path(&self) -> &Vec<ChildLocation> {
        self.path.path()
    }
    fn path_mut(&mut self) -> &mut Vec<ChildLocation> {
        self.path.path_mut()
    }
}
/// access to a rooted path pointing to a descendant
pub trait HasRootedRolePath<R: PathRole>: HasRolePath<R> + RootedPath {
    fn rooted_role_path(&self) -> RootedRolePath<R, Self::Root>;
}

/// access to a rooted path pointing to a descendant
pub trait HasRolePath<R: PathRole> {
    fn role_path(&self) -> &RolePath<R>;
    fn role_path_mut(&mut self) -> &mut RolePath<R>;
}

pub trait HasMatchPaths: HasRolePath<Start> + HasRolePath<End> {
    fn into_paths(self) -> (RolePath<Start>, RolePath<End>);
}

pub trait HasSinglePath {
    fn single_path(&self) -> &[ChildLocation];
}
