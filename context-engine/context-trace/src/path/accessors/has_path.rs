use crate::{
    graph::vertex::location::child::ChildLocation,
    path::{
        accessors::role::{
            End,
            PathRole,
            Start,
        },
        structs::{
            role_path::RolePath,
            rooted::{
                role_path::RootedRolePath,
                root::RootedPath,
            },
        },
    },
};
use auto_impl::auto_impl;

/// access to a rooted path pointing to a descendant
#[auto_impl(& mut)]
pub trait HasPath<R> {
    fn path(&self) -> &Vec<ChildLocation>;
    fn path_mut(&mut self) -> &mut Vec<ChildLocation>;
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
