use crate::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RootedRangePath<Root: PathRoot> {
    pub root: Root,
    pub start: RolePath<Start>,
    pub end: RolePath<End>,
}
pub type SearchPath = RootedRangePath<IndexRoot>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IndexRoot {
    pub location: PatternLocation,
    pub pos: TokenLocation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RootedSplitPath<Root: PathRoot> {
    pub root: Root,
    pub sub_path: SubPath,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RootedRolePath<R: PathRole, Root: PathRoot = IndexRoot> {
    pub split_path: RootedSplitPath<Root>,
    pub _ty: std::marker::PhantomData<R>,
}
impl From<SearchPath> for RootedRolePath<Start, IndexRoot> {
    fn from(path: SearchPath) -> Self {
        Self {
            split_path: RootedSplitPath {
                root: path.root,
                sub_path: path.start.sub_path,
            },
            _ty: Default::default(),
        }
    }
}
impl From<SearchPath> for RootedRolePath<End, IndexRoot> {
    fn from(path: SearchPath) -> Self {
        Self {
            split_path: RootedSplitPath {
                root: path.root,
                sub_path: path.end.sub_path,
            },
            _ty: Default::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
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
pub trait PathRoot {
}
impl PathRoot for Pattern {
}
impl PathRoot for IndexRoot {
}

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
impl<R: PathRole, Root: PathRoot> RootedPath for RootedRolePath<R, Root> {
    type Root = Root;
    fn path_root(&self) -> &Self::Root {
        &self.split_path.root
    }
}