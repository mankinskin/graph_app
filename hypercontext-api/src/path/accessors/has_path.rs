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

//impl HasPath<End> for OverlapPrimer {
//    fn path(&self) -> &Vec<ChildLocation> {
//        if self.exit == 0 {
//            self.end.borrow()
//        } else {
//            self.context.end.borrow()
//        }
//    }
//    fn path_mut(&mut self) -> &mut Vec<ChildLocation> {
//        if self.exit == 0 {
//            self.end.borrow_mut()
//        } else {
//            self.context.end.borrow_mut()
//        }
//    }
//}
/// access to a rooted path pointing to a descendant
pub trait IntoRootedRolePath<R: PathRole>: HasRolePath<R> + RootedPath {
    fn into_rooted_role_path(&self) -> RootedRolePath<R, Self::Root>;
}

/// access to a rooted path pointing to a descendant
pub trait HasRolePath<R: PathRole> {
    fn role_path(&self) -> &RolePath<R>;
    fn role_path_mut(&mut self) -> &mut RolePath<R>;
    fn num_path_segments(&self) -> usize {
        self.role_path().num_path_segments()
    }
}

pub trait HasMatchPaths: HasRolePath<Start> + HasRolePath<End> {
    fn into_paths(self) -> (RolePath<Start>, RolePath<End>);
    fn num_path_segments(&self) -> usize {
        HasRolePath::<Start>::role_path(self).num_path_segments()
            + HasRolePath::<End>::role_path(self).num_path_segments()
    }
    fn min_path_segments(&self) -> usize {
        HasRolePath::<Start>::role_path(self)
            .num_path_segments()
            .min(HasRolePath::<End>::role_path(self).num_path_segments())
    }
}

pub trait HasSinglePath {
    fn single_path(&self) -> &[ChildLocation];
}
