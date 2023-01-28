use crate::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RootedRangePath<Root: PathRoot> {
    pub root: Root,
    pub start: RolePath<Start>,
    pub end: RolePath<End>,
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RootedSplitPath<Root: PathRoot> {
    pub root: Root,
    pub path: SubPath,
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RootedRolePath<R: PathRole, Root: PathRoot = PatternLocation> {
    pub path: RootedSplitPath<Root>,
    pub _ty: std::marker::PhantomData<R>,
}
impl From<SearchPath> for RootedRolePath<Start, PatternLocation> {
    fn from(path: SearchPath) -> Self {
        Self {
            path: RootedSplitPath {
                root: path.root,
                path: path.start.path,
            },
            _ty: Default::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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
impl PathRoot for PatternLocation {
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
    type Root = PatternLocation;
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
        &self.path.root
    }
}
pub trait RangePath: RootedPath {
    fn new_range(root: Self::Root, entry: usize, exit: usize) -> Self;
}
impl RangePath for QueryRangePath {
    fn new_range(root: Self::Root, entry: usize, exit: usize) -> Self {
        Self {
            root,
            start: SubPath::new(entry).into(),
            end: SubPath::new(exit).into(),
        }
    }
}
impl RangePath for SearchPath {
    fn new_range(root: Self::Root, entry: usize, exit: usize) -> Self {
        Self {
            root,
            start: SubPath::new(entry).into(),
            end: SubPath::new(exit).into(),
        }
    }
}