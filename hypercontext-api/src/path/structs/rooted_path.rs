use std::ops::Deref;

use derive_more::derive::From;

use crate::{
    graph::vertex::{
        location::{
            child::ChildLocation,
            pattern::{IntoPatternLocation, PatternLocation},
        }, pattern::Pattern
    }, path::{
        accessors::role::{
            End,
            PathRole,
            Start,
        },
        structs::{
            query_range_path::QueryRangePath,
            role_path::RolePath,
        },
    }
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

#[derive(Clone, Debug, PartialEq, Eq, From)]
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

impl<R: PathRole> RootedRolePath<R> {
    pub fn new(first: ChildLocation) -> Self {
        Self {
            role_path: RolePath::from(SubPath::new(first.sub_index)),
            root: IndexRoot::from(first.into_pattern_location()),
        }
    }
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
                },
                _ty: Default::default(),
            },
        }
    }
}
impl<R: PathRoot> RootedRolePath<End, R> {
    pub fn into_range(
        self,
        entry: usize,
    ) -> RootedRangePath<R> {
        RootedRangePath {
            root: self.root,
            start: RolePath {
                sub_path: SubPath {
                    root_entry: entry,
                    path: vec![],
                },
                _ty: Default::default(),
            },
            end: self.role_path,
        }
    }
}

impl<R: PathRoot> From<RootedRangePath<R>> for RootedRolePath<Start, R> {
    fn from(path: RootedRangePath<R>) -> Self {
        Self {
            root: path.root,
            role_path: path.start,
        }
    }
}
impl<R: PathRoot> From<RootedRangePath<R>> for RootedRolePath<End, R> {
    fn from(path: RootedRangePath<R>) -> Self {
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
