use std::ops::Deref;

use crate::{
    graph::vertex::location::child::ChildLocation,
    path::{
        accessors::role::{
            End,
            PathRole,
            Start,
        },
        mutators::adapters::from_advanced::FromAdvanced,
        structs::{
            rooted::root::PathRoot,
            sub_path::SubPath,
        },
    },
    traversal::traversable::Traversable,
};

use super::rooted::{
    index_range::IndexRangePath,
    role_path::RootedRolePath,
};

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct RolePath<R: PathRole> {
    pub sub_path: SubPath,
    pub _ty: std::marker::PhantomData<R>,
}

impl<R: PathRole> RolePath<R> {
    pub fn num_path_segments(&self) -> usize {
        self.path().len()
    }
    pub fn path(&self) -> &Vec<ChildLocation> {
        &self.sub_path.path
    }
    pub fn path_mut(&mut self) -> &mut Vec<ChildLocation> {
        &mut self.sub_path.path
    }
    pub fn into_rooted<Root: PathRoot>(
        self,
        root: Root,
    ) -> RootedRolePath<R, Root> {
        RootedRolePath {
            root,
            role_path: self,
        }
    }
}

impl<R: PathRole> Deref for RolePath<R> {
    type Target = SubPath;
    fn deref(&self) -> &Self::Target {
        &self.sub_path
    }
}

impl From<IndexRangePath> for RolePath<Start> {
    fn from(p: IndexRangePath) -> Self {
        p.start
    }
}

impl<R: PathRole> From<SubPath> for RolePath<R> {
    fn from(sub_path: SubPath) -> Self {
        Self {
            sub_path,
            _ty: Default::default(),
        }
    }
}

impl From<IndexRangePath> for RolePath<End> {
    fn from(p: IndexRangePath) -> Self {
        p.end
    }
}
//impl<R> WideMut for RolePath<R> {
//    fn width_mut(&mut self) -> &mut usize {
//        &mut self.width
//    }
//}

impl FromAdvanced<IndexRangePath> for RolePath<Start> {
    fn from_advanced<Trav: Traversable>(
        path: IndexRangePath,
        _trav: &Trav,
    ) -> Self {
        path.start
    }
}
