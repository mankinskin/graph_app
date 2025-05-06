pub mod index_range;
pub mod pattern_range;
pub mod role_path;
pub mod root;
pub mod split_path;

use root::{
    PathRoot,
    RootedPath,
};
use split_path::RootedSplitPathRef;

use crate::path::{
    accessors::role::{
        End,
        Start,
    },
    structs::role_path::RolePath,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RootedRangePath<Root: PathRoot> {
    pub root: Root,
    pub start: RolePath<Start>,
    pub end: RolePath<End>,
}
impl<R: PathRoot> RootedPath for RootedRangePath<R> {
    type Root = R;
    fn path_root(&self) -> Self::Root {
        self.root.clone()
    }
}

impl<R: PathRoot> RootedRangePath<R> {
    pub fn start_path(&self) -> RootedSplitPathRef<'_, R> {
        RootedSplitPathRef {
            root: &self.root,
            sub_path: &self.start.sub_path,
        }
    }
    pub fn end_path(&self) -> RootedSplitPathRef<'_, R> {
        RootedSplitPathRef {
            root: &self.root,
            sub_path: &self.end.sub_path,
        }
    }
}
