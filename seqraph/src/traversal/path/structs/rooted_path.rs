use std::ops::Deref;

use crate::{
    traversal::path::{
        accessors::role::{
            End,
            PathRole,
            Start,
        },
        structs::{
            query_range_path::QueryRangePath,
            role_path::RolePath,
        },
    },
    vertex::{
        location::{
            child::ChildLocation,
            pattern::PatternLocation,
        },
        pattern::Pattern,
    },
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RootedRangePath<Root: PathRoot> {
    pub root: Root,
    pub start: RolePath<Start>,
    pub end: RolePath<End>,
}

impl<R: PathRoot> RootedRangePath<R> {
    pub fn end_path(&self) -> RootedSplitPathRef<'_, R> {
        RootedSplitPathRef {
            root: &self.root,
            sub_path: &self.end.sub_path,
        }
    }
}

pub type SearchPath = RootedRangePath<IndexRoot>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IndexRoot {
    pub location: PatternLocation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RootedSplitPath<Root: PathRoot = IndexRoot> {
    pub root: Root,
    pub sub_path: SubPath,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RootedSplitPathRef<'a, Root: PathRoot = IndexRoot> {
    pub root: &'a Root,
    pub sub_path: &'a SubPath,
}

impl<'a, R: PathRoot> From<&'a RootedSplitPath<R>> for RootedSplitPathRef<'a, R> {
    fn from(value: &'a RootedSplitPath<R>) -> Self {
        Self {
            root: &value.root,
            sub_path: &value.sub_path,
        }
    }
}

impl<'a, R: PathRole, Root: PathRoot> From<&'a RootedRolePath<R, Root>>
for RootedSplitPathRef<'a, Root>
{
    fn from(value: &'a RootedRolePath<R, Root>) -> Self {
        Self {
            root: &value.root,
            sub_path: &value.role_path.sub_path,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RootedRolePath<R: PathRole, Root: PathRoot = IndexRoot> {
    pub root: Root,
    pub role_path: RolePath<R>,
}

impl<R: PathRoot> RootedRolePath<Start, R> {
    pub fn into_range(
        self,
        exit: usize,
    ) -> RootedRangePath<R> {
        RootedRangePath {
            root: self.root,
            start: self.role_path,
            end: RolePath {
                sub_path: SubPath {
                    root_entry: exit,
                    path: vec![],
                }
                    .into(),
                _ty: Default::default(),
            },
        }
    }
}

impl From<SearchPath> for RootedRolePath<Start, IndexRoot> {
    fn from(path: SearchPath) -> Self {
        Self {
            root: path.root,
            role_path: path.start,
        }
    }
}

impl From<SearchPath> for RootedRolePath<End, IndexRoot> {
    fn from(path: SearchPath) -> Self {
        Self {
            root: path.root,
            role_path: path.end,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SubPath {
    pub root_entry: usize,
    pub path: Vec<ChildLocation>,
}

impl Deref for SubPath {
    type Target = Vec<ChildLocation>;
    fn deref(&self) -> &Self::Target {
        &self.path
    }
}

impl SubPath {
    pub fn new(root_entry: usize) -> Self {
        Self {
            root_entry,
            path: vec![],
        }
    }
}

pub trait PathRoot {}

impl PathRoot for Pattern {}

impl PathRoot for IndexRoot {}

pub trait RootedPath {
    type Root: PathRoot;
    fn path_root(&self) -> &Self::Root;
}

impl RootedPath for QueryRangePath {
    type Root = Pattern;
    fn path_root(&self) -> &Self::Root {
        &self.root
    }
}

impl RootedPath for SearchPath {
    type Root = IndexRoot;
    fn path_root(&self) -> &Self::Root {
        &self.root
    }
}

impl<R: PathRoot> RootedPath for RootedSplitPath<R> {
    type Root = R;
    fn path_root(&self) -> &Self::Root {
        &self.root
    }
}

impl<R: PathRoot> RootedPath for RootedSplitPathRef<'_, R> {
    type Root = R;
    fn path_root(&self) -> &Self::Root {
        self.root
    }
}

impl<R: PathRole, Root: PathRoot> RootedPath for RootedRolePath<R, Root> {
    type Root = Root;
    fn path_root(&self) -> &Self::Root {
        &self.root
    }
}
