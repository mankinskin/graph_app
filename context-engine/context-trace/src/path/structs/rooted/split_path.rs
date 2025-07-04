use derive_more::Deref;

use crate::{
    graph::vertex::location::child::ChildLocation,
    impl_root,
    path::{
        accessors::{
            child::{
                RootChildIndex,
                root::GraphRootChild,
            },
            role::PathRole,
            root::GraphRootPattern,
        },
        structs::sub_path::SubPath,
    },
};

use super::{
    role_path::RootedRolePath,
    root::{
        IndexRoot,
        PathRoot,
        RootedPath,
    },
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RootedSplitPath<Root: PathRoot = IndexRoot> {
    pub root: Root,
    pub sub_path: SubPath,
}

#[derive(Debug, Clone, PartialEq, Eq, Deref)]
pub struct RootedSplitPathRef<'a, Root: PathRoot = IndexRoot> {
    pub root: &'a Root,
    #[deref]
    pub sub_path: &'a SubPath,
}

impl<'a, R: PathRoot> From<&'a RootedSplitPath<R>>
    for RootedSplitPathRef<'a, R>
{
    fn from(value: &'a RootedSplitPath<R>) -> Self {
        Self {
            root: &value.root,
            sub_path: &value.sub_path,
        }
    }
}
impl<R: PathRoot> RootedPath for RootedSplitPath<R> {
    type Root = R;
    fn path_root(&self) -> Self::Root {
        self.root.clone()
    }
}

impl<R: PathRoot> RootedPath for RootedSplitPathRef<'_, R> {
    type Root = R;
    fn path_root(&self) -> Self::Root {
        self.root.clone()
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

impl_root! { GraphRootPattern for RootedSplitPath<IndexRoot>, self => self.root.location.clone() }
impl_root! { GraphRootPattern for RootedSplitPathRef<'_, IndexRoot>, self => self.root.location.clone() }
impl_root! { GraphRoot for RootedSplitPath<IndexRoot>, self => self.root.location.parent.clone() }
impl_root! { GraphRoot for RootedSplitPathRef<'_, IndexRoot>, self => self.root.location.parent.clone() }

impl<R: PathRole, Root: PathRoot> RootChildIndex<R> for RootedSplitPath<Root> {
    fn root_child_index(&self) -> usize {
        RootChildIndex::<R>::root_child_index(&self.sub_path)
    }
}

impl<R: PathRole, Root: PathRoot> RootChildIndex<R>
    for RootedSplitPathRef<'_, Root>
{
    fn root_child_index(&self) -> usize {
        RootChildIndex::<R>::root_child_index(self.sub_path)
    }
}

impl<R: PathRole> GraphRootChild<R> for RootedSplitPath<IndexRoot> {
    fn root_child_location(&self) -> ChildLocation {
        self.path_root()
            .location
            .to_child_location(self.sub_path.root_entry)
    }
}

impl<R: PathRole> GraphRootChild<R> for RootedSplitPathRef<'_, IndexRoot> {
    fn root_child_location(&self) -> ChildLocation {
        self.path_root()
            .location
            .to_child_location(self.sub_path.root_entry)
    }
}

impl_root! { RootPattern for RootedSplitPath<IndexRoot>, self, trav => GraphRootPattern::graph_root_pattern::<G>(self, trav) }
impl_root! { RootPattern for RootedSplitPathRef<'_, IndexRoot>, self, trav => GraphRootPattern::graph_root_pattern::<G>(self, trav) }
