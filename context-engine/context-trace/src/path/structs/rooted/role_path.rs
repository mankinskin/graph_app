use std::borrow::Borrow;

use crate::{
    graph::{
        getters::ErrorReason,
        vertex::{
            child::Child,
            location::{
                child::ChildLocation,
                pattern::{
                    IntoPatternLocation,
                    PatternLocation,
                },
            },
            pattern::{
                IntoPattern,
                Pattern,
            },
        },
    },
    impl_child,
    path::{
        accessors::{
            child::{
                LeafChild,
                PathChild,
                RootChildIndex,
                RootChildIndexMut,
                root::{
                    GraphRootChild,
                    PatternRootChild,
                },
            },
            has_path::{
                HasPath,
                HasRolePath,
                HasSinglePath,
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
            query_range_path::FoldablePath,
            role_path::{
                CalcOffset,
                RolePath,
            },
            sub_path::SubPath,
        },
    },
    trace::has_graph::HasGraph,
};

use super::{
    RootedRangePath,
    pattern_range::PatternRangePath,
    root::{
        IndexRoot,
        PathRoot,
        RootedPath,
    },
};
use crate::graph::vertex::wide::Wide;

pub trait CalcWidth: CalcOffset + RootedPath {
    fn calc_width<G: HasGraph>(
        &self,
        trav: G,
    ) -> usize;
}
impl<Role: PathRole, Root: PathRoot> CalcOffset for RootedRolePath<Role, Root> {
    fn calc_offset<G: HasGraph>(
        &self,
        trav: G,
    ) -> usize {
        self.role_path.calc_offset(trav)
    }
}
impl<Role: PathRole, Root: PathRoot> CalcWidth for RootedRolePath<Role, Root>
where
    Self: LeafChild<Role>,
{
    // TODO: Make offset side relative
    fn calc_width<G: HasGraph>(
        &self,
        trav: G,
    ) -> usize {
        self.calc_offset(&trav) + self.leaf_child(&trav).width()
    }
}
pub type IndexRolePath<R> = RootedRolePath<R, IndexRoot>;
pub type PatternRolePath<R> = RootedRolePath<R, Pattern>;

pub type IndexStartPath = IndexRolePath<Start>;
pub type IndexEndPath = IndexRolePath<End>;
pub type PatternStartPath = PatternRolePath<Start>;
pub type PatternEndPath = PatternRolePath<End>;

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

impl<R: PathRole, Root: PathRoot> RootChildIndexMut<R>
    for RootedRolePath<R, Root>
{
    fn root_child_index_mut(&mut self) -> &mut usize {
        self.role_path.root_child_index_mut()
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
    fn path_root(&self) -> Self::Root {
        self.root.clone()
    }
}

impl<R: PathRole> PathChild<R> for RolePath<R> {}

impl_child! {
    RootChild for IndexRolePath<R>, self,
    trav => trav.graph().expect_child_at(
            self.path_root().location.to_child_location(
                RootChildIndex::<R>::root_child_index(&self.role_path)
            )
        )
        .clone()
}
impl<R: PathRole> GraphRootChild<R> for RootedRolePath<R, IndexRoot> {
    fn root_child_location(&self) -> ChildLocation {
        self.path_root()
            .location
            .to_child_location(self.role_path.sub_path.root_entry)
    }
}
impl<R: PathRole, Root: PathRoot> RootChildIndex<R>
    for RootedRolePath<R, Root>
{
    fn root_child_index(&self) -> usize {
        RootChildIndex::<R>::root_child_index(&self.role_path)
    }
}

impl<R: PathRole> GraphRoot for RootedRolePath<R, IndexRoot> {
    fn root_parent(&self) -> Child {
        self.root.location.parent.clone()
    }
}

impl<R: PathRole> GraphRootPattern for RootedRolePath<R, IndexRoot> {
    fn root_pattern_location(&self) -> PatternLocation {
        self.root.location.clone()
    }
}

impl<R: PathRole, Root: PathRoot> HasSinglePath for RootedRolePath<R, Root> {
    fn single_path(&self) -> &[ChildLocation] {
        self.role_path.sub_path.path.borrow()
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

impl<Role: PathRole, Root: PathRoot> RootPattern
    for RootedRolePath<Role, Root>
{
    fn root_pattern<'a: 'g, 'b: 'g, 'g, G: HasGraph + 'a>(
        &'b self,
        trav: &'g G::Guard<'a>,
    ) -> &'g Pattern {
        self.root.root_pattern::<G>(trav)
    }
}

//impl_root! { <R: PathRole> PatternRoot for PatternRolePath<R>, self => self.root.borrow() }

impl RootChildIndex<Start> for PatternEndPath {
    fn root_child_index(&self) -> usize {
        0
    }
}
//impl<R: PathRole> PathChild<R> for PatternRolePath<R> where
//    Self: HasPath<R> + PatternRootChild<R>
//{
//}
//
//impl<R: PathRole> PatternRootChild<R> for PatternRolePath<R> where
//    PatternEndPath: RootChildIndex<R>
//{
//}

impl HasPath<End> for PatternEndPath {
    fn path(&self) -> &Vec<ChildLocation> {
        self.role_path.path()
    }
    fn path_mut(&mut self) -> &mut Vec<ChildLocation> {
        self.role_path.path_mut()
    }
}
//impl PathChild<Start> for PatternEndPath {
//    fn path_child_location(&self) -> Option<ChildLocation> {
//        None
//    }
//    fn path_child<G: HasGraph>(
//        &self,
//        trav: &G,
//    ) -> Option<Child> {
//        Some(self.root_child())
//    }
//}
//impl_child! { RootChild for PatternRolePath<R>, self, _trav => self.pattern_root_child() }

impl_child! { RootChild for PatternRolePath<R>, self, _trav => self.pattern_root_child() }

impl FoldablePath for PatternEndPath {
    fn to_range_path(self) -> PatternRangePath {
        self.into_range(0)
    }
    fn complete(query: impl IntoPattern) -> Self {
        let pattern = query.into_pattern();
        Self {
            role_path: RolePath::from(SubPath::new(pattern.len() - 1)),
            root: pattern,
        }
    }
    fn new_directed<D>(query: Pattern) -> Result<Self, (ErrorReason, Self)> {
        let pattern = query.into_pattern();
        let len = pattern.len();
        let p = Self {
            role_path: RolePath::from(SubPath::new(0)),
            root: pattern,
        };
        match len {
            0 => Err((ErrorReason::EmptyPatterns, p)),
            1 => Err((ErrorReason::SingleIndex(*p.root.first().unwrap()), p)),
            _ => Ok(p),
        }
    }
}
