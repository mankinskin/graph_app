use crate::{
    graph::vertex::{
        child::Child,
        location::{
            child::ChildLocation,
            pattern::{
                IntoPatternLocation,
                PatternLocation,
            },
        },
        pattern::Pattern,
    },
    impl_child,
    path::{
        accessors::{
            child::{
                pos::RootChildPos,
                root::GraphRootChild,
                PathChild,
            },
            role::{
                End,
                PathRole,
                Start,
            },
            root::{
                GraphRoot,
                GraphRootPattern,
                RootPattern,
            },
        },
        structs::{
            role_path::RolePath,
            sub_path::SubPath,
        },
    },
    traversal::traversable::Traversable,
};

use super::{
    root::{
        IndexRoot,
        PathRoot,
        RootedPath,
    },
    RootedRangePath,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RootedRolePath<R: PathRole, Root: PathRoot> {
    pub root: Root,
    pub role_path: RolePath<R>,
}

impl<R: PathRole> RootedRolePath<R, IndexRoot> {
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

impl<R: PathRole, Root: PathRoot> RootedPath for RootedRolePath<R, Root> {
    type Root = Root;
    fn path_root(&self) -> &Self::Root {
        &self.root
    }
}

impl<R: PathRole> PathChild<R> for RolePath<R> {}

impl_child! {
    RootChild for RootedRolePath<R, IndexRoot>, self,
    trav => trav.graph().expect_child_at(self.path_root().location.to_child_location(RootChildPos::<R>::root_child_pos(&self.role_path)))
}
impl<R: PathRole> GraphRootChild<R> for RootedRolePath<R, IndexRoot> {
    fn root_child_location(&self) -> ChildLocation {
        self.path_root()
            .location
            .to_child_location(self.role_path.sub_path.root_entry)
    }
}

impl<R: PathRole> GraphRoot for RootedRolePath<R, IndexRoot> {
    fn root_parent(&self) -> Child {
        self.root.location.parent
    }
}

impl<R: PathRole> GraphRootPattern for RootedRolePath<R, IndexRoot> {
    fn root_pattern_location(&self) -> PatternLocation {
        self.root.location
    }
}
impl<R: PathRole> RootPattern for RootedRolePath<R, IndexRoot> {
    fn root_pattern<'a: 'g, 'b: 'g, 'g, Trav: Traversable + 'a>(
        &'b self,
        trav: &'g Trav::Guard<'a>,
    ) -> &'g Pattern {
        GraphRootPattern::graph_root_pattern::<Trav>(self, trav)
    }
}
