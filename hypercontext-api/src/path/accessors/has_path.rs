use crate::{
    graph::vertex::location::child::ChildLocation,
    path::structs::{
        query_range_path::FoldablePath,
        rooted::{
            role_path::RootedRolePath,
            root::PathRoot,
        },
    },
    traversal::state::cursor::PathCursor,
};
use crate::{
    path::{
        accessors::role::{
            End,
            PathRole,
            Start,
        },
        structs::role_path::RolePath,
    },
    //traversal::state::query::RangeCursor,
};
use auto_impl::auto_impl;
use std::borrow::Borrow;

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
impl<R> HasPath<R> for RolePath<R> {
    fn path(&self) -> &Vec<ChildLocation> {
        &self.path
    }
    fn path_mut(&mut self) -> &mut Vec<ChildLocation> {
        &mut self.sub_path.path
    }
}

/// access to a rooted path pointing to a descendant
pub trait HasRolePath<R> {
    fn role_path(&self) -> &RolePath<R>;
    fn role_path_mut(&mut self) -> &mut RolePath<R>;
    fn num_path_segments(&self) -> usize {
        self.role_path().num_path_segments()
    }
}

impl<R> HasRolePath<R> for RolePath<R> {
    fn role_path(&self) -> &RolePath<R> {
        self
    }
    fn role_path_mut(&mut self) -> &mut RolePath<R> {
        self
    }
}

impl<R: PathRole, Root: PathRoot> HasRolePath<R> for RootedRolePath<R, Root> {
    fn role_path(&self) -> &RolePath<R> {
        &self.role_path
    }
    fn role_path_mut(&mut self) -> &mut RolePath<R> {
        &mut self.role_path
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

impl<R> HasSinglePath for RolePath<R> {
    fn single_path(&self) -> &[ChildLocation] {
        self.path().borrow()
    }
}

impl<R: PathRole, Root: PathRoot> HasSinglePath for RootedRolePath<R, Root> {
    fn single_path(&self) -> &[ChildLocation] {
        self.role_path.sub_path.path.borrow()
    }
}
