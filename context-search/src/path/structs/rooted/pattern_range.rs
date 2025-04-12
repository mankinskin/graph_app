use std::{
    borrow::Borrow,
    ops::ControlFlow,
};

use crate::{
    direction::{
        pattern::PatternDirection,
        Right,
    },
    graph::{
        getters::ErrorReason,
        kind::{
            BaseGraphKind,
            GraphKind,
        },
        vertex::{
            location::child::ChildLocation,
            pattern::{
                IntoPattern,
                Pattern,
            },
        },
    },
    impl_child,
    impl_root,
    path::{
        accessors::{
            child::{
                root::PatternRootChild,
                PathChild,
                RootChildPos,
                RootChildPosMut,
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
            move_path::root::MoveRootPos,
            pop::PathPop,
        },
        structs::{
            query_range_path::{
                FoldablePath,
                RangePath,
            },
            role_path::RolePath,
            sub_path::SubPath,
        },
    },
    traversal::traversable::{
        TravDir,
        Traversable,
    },
};

use super::{
    role_path::RootedRolePath,
    root::{
        PathRoot,
        RootedPath,
    },
    RootedRangePath,
};

pub type PatternRangePath = RootedRangePath<Pattern>;

impl RootChildPosMut<End> for PatternRangePath {
    fn root_child_pos_mut(&mut self) -> &mut usize {
        &mut self.end.sub_path.root_entry
    }
}

impl<P: IntoPattern> From<P> for PatternRangePath {
    fn from(p: P) -> Self {
        let p = p.into_pattern();
        let entry = <BaseGraphKind as GraphKind>::Direction::head_index(&p.borrow());
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

impl RootChildPos<Start> for PatternRangePath {
    fn root_child_pos(&self) -> usize {
        self.start.root_entry
    }
}
impl RootChildPos<End> for PatternRangePath {
    fn root_child_pos(&self) -> usize {
        self.end.root_entry
    }
}

impl MoveRootPos<Right, End> for PatternRangePath {
    fn move_root_pos<Trav: Traversable>(
        &mut self,
        _trav: &Trav,
    ) -> ControlFlow<()> {
        if let Some(next) = TravDir::<Trav>::index_next(RootChildPos::<End>::root_child_pos(self)) {
            *self.root_child_pos_mut() = next;
            ControlFlow::Continue(())
        } else {
            ControlFlow::Break(())
        }
    }
}

impl<R: PathRole> PathChild<R> for PatternRangePath where Self: HasPath<R> + PatternRootChild<R> {}

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
impl<R: PathRole> HasRootedRolePath<R> for PatternRangePath
where
    Self: HasRolePath<R> + RootedPath,
{
    fn rooted_role_path(&self) -> RootedRolePath<R, Self::Root> {
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

impl_child! { RootChild for PatternRangePath, self, _trav => self.pattern_root_child() }

impl<R> PatternRootChild<R> for PatternRangePath where Self: RootChildPos<R> {}

impl FoldablePath for PatternRangePath {
    fn to_range_path(self) -> PatternRangePath {
        self
    }
    fn complete(query: impl IntoPattern) -> Self {
        let query = query.into_pattern();
        let len = query.len();
        Self::new_range(query, 0, len - 1)
    }
    fn new_directed<D: PatternDirection>(query: Pattern) -> Result<Self, (ErrorReason, Self)> {
        let entry = D::head_index(&query);
        let query = query.into_pattern();
        let len = query.len();
        let query = Self::new_range(query, entry, entry);
        match len {
            0 => Err((ErrorReason::EmptyPatterns, query)),
            1 => Err((
                ErrorReason::SingleIndex(*query.root.first().unwrap()),
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
