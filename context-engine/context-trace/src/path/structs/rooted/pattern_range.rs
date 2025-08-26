use std::{
    borrow::Borrow,
    ops::ControlFlow,
};

use super::{
    RootedRangePath,
    role_path::{
        CalcWidth,
        PatternRolePath,
        RootedRolePath,
    },
    root::{
        PathRoot,
        RootedPath,
    },
};
use crate::{
    direction::{
        Right,
        pattern::PatternDirection,
    },
    graph::{
        getters::{
            ErrorReason,
            IndexWithPath,
        },
        kind::{
            BaseGraphKind,
            GraphKind,
        },
        vertex::{
            location::child::ChildLocation,
            pattern::{
                IntoPattern,
                Pattern,
                pattern_width,
            },
            wide::Wide,
        },
    },
    impl_child,
    impl_root,
    path::{
        accessors::{
            child::{
                LeafChild,
                PathChild,
                RootChildIndex,
                RootChildIndexMut,
                root::PatternRootChild,
            },
            has_path::{
                HasPath,
                HasRolePath,
                HasRootedRolePath,
            },
            role::{
                End,
                PathRole,
                Start,
            },
            root::PatternRoot,
        },
        mutators::{
            move_path::root::MoveRootIndex,
            pop::PathPop,
        },
        structs::{
            query_range_path::{
                FoldablePath,
                RangePath,
            },
            role_path::{
                CalcOffset,
                RolePath,
            },
            sub_path::SubPath,
        },
    },
    trace::has_graph::{
        HasGraph,
        TravDir,
    },
};

pub type PatternRangePath = RootedRangePath<Pattern>;
pub type PatternPrefixPath = RootedRolePath<Start, Pattern>;
pub type PatternPostfixPath = RootedRolePath<End, Pattern>;

impl RootChildIndexMut<End> for PatternRangePath {
    fn root_child_index_mut(&mut self) -> &mut usize {
        &mut self.end.sub_path.root_entry
    }
}

impl<P: IntoPattern> From<P> for PatternRangePath {
    fn from(p: P) -> Self {
        let p = p.into_pattern();
        let entry =
            <BaseGraphKind as GraphKind>::Direction::head_index(p.borrow());
        RootedRangePath {
            root: p,
            start: SubPath::new(entry).into(),
            end: SubPath::new(entry).into(),
        }
    }
}
impl RangePath for PatternRangePath {
    fn new_range(
        root: Self::Root,
        entry: usize,
        exit: usize,
    ) -> Self {
        Self {
            root,
            start: SubPath::new(entry).into(),
            end: SubPath::new(exit).into(),
        }
    }
}

impl_root! { RootPattern for PatternRangePath, self, _trav => PatternRoot::pattern_root_pattern(self) }
impl_root! { PatternRoot for PatternRangePath, self => self.root.borrow() }
impl_root! { <Role: PathRole> PatternRoot for PatternRolePath<Role>, self => self.root.borrow() }

impl RootChildIndex<Start> for PatternRangePath {
    fn root_child_index(&self) -> usize {
        self.start.root_entry
    }
}
impl RootChildIndex<End> for PatternRangePath {
    fn root_child_index(&self) -> usize {
        self.end.root_entry
    }
}

impl MoveRootIndex<Right, End> for PatternRangePath {
    fn move_root_index<G: HasGraph>(
        &mut self,
        _trav: &G,
    ) -> ControlFlow<()> {
        if let Some(next) = TravDir::<G>::index_next(
            RootChildIndex::<End>::root_child_index(self),
        ) {
            *self.root_child_index_mut() = next;
            ControlFlow::Continue(())
        } else {
            ControlFlow::Break(())
        }
    }
}

impl<R: PathRole> PathChild<R> for PatternRolePath<R> where
    Self: HasPath<R> + PatternRootChild<R>
{
}
impl<R: PathRole> PathChild<R> for PatternRangePath where
    Self: HasPath<R> + PatternRootChild<R>
{
}

impl<R: PathRole> HasPath<R> for PatternRangePath
where
    Self: HasRolePath<R>,
{
    fn path(&self) -> &Vec<ChildLocation> {
        HasRolePath::<R>::role_path(self).path()
    }
    fn path_mut(&mut self) -> &mut Vec<ChildLocation> {
        HasRolePath::<R>::role_path_mut(self).path_mut()
    }
}
impl<Role: PathRole, Root: PathRoot + Clone> HasRootedRolePath<Role>
    for RootedRangePath<Root>
where
    Self: HasRolePath<Role> + RootedPath<Root = Root>,
{
    fn rooted_role_path(&self) -> RootedRolePath<Role, Self::Root> {
        self.role_path()
            .clone()
            .into_rooted(self.path_root().clone())
    }
}
impl HasRolePath<Start> for PatternRangePath {
    fn role_path(&self) -> &RolePath<Start> {
        &self.start
    }
    fn role_path_mut(&mut self) -> &mut RolePath<Start> {
        &mut self.start
    }
}
impl HasRolePath<End> for PatternRangePath {
    fn role_path(&self) -> &RolePath<End> {
        &self.end
    }
    fn role_path_mut(&mut self) -> &mut RolePath<End> {
        &mut self.end
    }
}

impl<R: PathRole> PatternRootChild<R> for PatternRolePath<R> where
    Self: RootChildIndex<R>
{
}
impl<R: PathRole> PatternRootChild<R> for PatternRangePath where
    Self: RootChildIndex<R>
{
}

impl_child! { RootChild for PatternRangePath, self, _trav =>
       *self.root.get(self.role_root_child_index::<R>()).unwrap()
}
use crate::path::RolePathUtils;
impl<Root: PathRoot> CalcOffset for RootedRangePath<Root> {
    fn calc_offset<G: HasGraph>(
        &self,
        trav: G,
    ) -> usize {
        let outer_offsets =
            self.start.calc_offset(&trav) + self.end.calc_offset(&trav);
        let graph = trav.graph();
        let pattern = self.root.root_pattern::<G>(&graph);
        let entry = self.start.sub_path.root_entry;
        let exit = self.end.sub_path.root_entry;
        let inner_offset = if entry < exit {
            pattern_width(&pattern[entry + 1..exit])
        } else {
            0
        };
        inner_offset + outer_offsets
    }
}
impl<Root: PathRoot> CalcWidth for RootedRangePath<Root>
where
    Self: LeafChild<Start> + LeafChild<End>,
{
    fn calc_width<G: HasGraph>(
        &self,
        trav: G,
    ) -> usize {
        self.calc_offset(&trav)
            + self.role_leaf_child::<Start, _>(&trav).width()
            + if self.role_root_child_index::<Start>()
                != self.role_root_child_index::<End>()
            {
                self.role_leaf_child::<End, _>(&trav).width()
            } else {
                0
            }
    }
}

impl FoldablePath for PatternRangePath {
    fn to_range_path(self) -> PatternRangePath {
        self
    }
    fn complete(query: impl IntoPattern) -> Self {
        let query = query.into_pattern();
        let len = query.len();
        Self::new_range(query, 0, len - 1)
    }
    fn new_directed<D: PatternDirection>(
        query: Pattern
    ) -> Result<Self, (ErrorReason, Self)> {
        let entry = D::head_index(&query);
        let query = query.into_pattern();
        let len = query.len();
        let query = Self::new_range(query, entry, entry);
        match len {
            0 => Err((ErrorReason::EmptyPatterns, query)),
            1 => Err((
                ErrorReason::SingleIndex(Box::new(IndexWithPath::from(
                    query.clone(),
                ))),
                query,
            )),
            _ => Ok(query),
        }
    }
}
impl<R: PathRoot> PathPop for RootedRangePath<R> {
    fn path_pop(&mut self) -> Option<ChildLocation> {
        self.end.path_pop()
    }
}
