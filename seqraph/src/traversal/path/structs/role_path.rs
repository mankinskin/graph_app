use std::ops::Deref;

use crate::traversal::path::{
    accessors::role::{
        End,
        PathRole,
        Start,
    },
    structs::rooted_path::{
        PathRoot,
        RootedRolePath,
        SearchPath,
        SubPath,
    },
};
use crate::graph::vertex::location::child::ChildLocation;

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct RolePath<R> {
    pub sub_path: SubPath,
    //pub child: Child,
    //pub width: usize,
    //pub token_pos: usize,
    pub _ty: std::marker::PhantomData<R>,
}

impl<R: PathRole> RolePath<R> {
    //pub fn get_child(&self) -> Child {
    //    self.child
    //}
    //pub fn into_context_path(self) -> Vec<ChildLocation> {
    //    self.path.path
    //}
    pub fn num_path_segments(&self) -> usize {
        self.path().len()
    }
    pub fn path(&self) -> &Vec<ChildLocation> {
        &self.sub_path.path
    }
    pub fn path_mut(&mut self) -> &mut Vec<ChildLocation> {
        &mut self.sub_path.path
    }
    //pub fn child_location(&self) -> ChildLocation {
    //    <Self as GraphRootChild<R>>::root_child_location(self)
    //}
    //pub fn child_location_mut(&mut self) -> &mut ChildLocation {
    //    <Self as GraphRootChild<R>>::root_child_location_mut(self)
    //}
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

impl<R> Deref for RolePath<R> {
    type Target = SubPath;
    fn deref(&self) -> &Self::Target {
        &self.sub_path
    }
}

impl From<SearchPath> for RolePath<Start> {
    fn from(p: SearchPath) -> Self {
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

impl From<SearchPath> for RolePath<End> {
    fn from(p: SearchPath) -> Self {
        p.end
    }
}
//impl<R> WideMut for RolePath<R> {
//    fn width_mut(&mut self) -> &mut usize {
//        &mut self.width
//    }
//}
