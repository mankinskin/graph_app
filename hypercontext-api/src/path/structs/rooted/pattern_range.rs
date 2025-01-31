use std::{
    borrow::Borrow,
    ops::ControlFlow,
};

use crate::{
    direction::{
        r#match::MatchDirection,
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
                pos::{
                    RootChildPos,
                    RootChildPosMut,
                },
                root::PatternRootChild,
                PathChild,
            },
            has_path::{
                HasPath,
                HasRolePath,
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
            sub_path::SubPath,
        },
    },
    traversal::traversable::{
        TravDir,
        Traversable,
    },
};

use super::{
    root::{
        PathRoot,
        RootedPath,
    },
    RootedRangePath,
};

pub type PatternRangePath = RootedRangePath<Pattern>;

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

impl_root! { RootPattern for PatternRangePath, self, _trav => PatternRoot::pattern_root_pattern(self) }
impl_root! { PatternRoot for PatternRangePath, self => self.root.borrow() }
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
impl<R: 'static> HasPath<R> for PatternRangePath
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
impl HasPath<End> for PatternRangePath {
    fn path(&self) -> &Vec<ChildLocation> {
        &self.end.path
    }
    fn path_mut(&mut self) -> &mut Vec<ChildLocation> {
        &mut self.end.sub_path.path
    }
}

impl HasPath<Start> for PatternRangePath {
    fn path(&self) -> &Vec<ChildLocation> {
        &self.start.path
    }
    fn path_mut(&mut self) -> &mut Vec<ChildLocation> {
        &mut self.start.sub_path.path
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
    fn new_directed<D: MatchDirection, P: IntoPattern>(
        query: P
    ) -> Result<Self, (ErrorReason, Self)> {
        let entry = D::head_index(&query.borrow());
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
impl<R: PathRoot> PathPop for RootedRangePath<R> {
    fn path_pop(&mut self) -> Option<ChildLocation> {
        self.end.path_pop()
    }
}

impl<R: PathRoot> RootedPath for RootedRangePath<R> {
    type Root = R;
    fn path_root(&self) -> &Self::Root {
        &self.root
    }
}
