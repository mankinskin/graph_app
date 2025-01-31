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
                pos::RootChildPos,
                root::PatternRootChild,
                PathChild,
            },
            has_path::HasPath,
            role::{
                End,
                PathRole,
                Start,
            },
        },
        mutators::move_path::{
            leaf::AdvanceLeaf,
            path::MovePath,
        },
        structs::{
            query_range_path::FoldablePath,
            role_path::RolePath,
            sub_path::SubPath,
        },
    },
    traversal::traversable::Traversable,
};

use super::{
    pattern_range::PatternRangePath,
    role_path::RootedRolePath,
};

pub type PatternPrefixPath = RootedRolePath<End, Pattern>;

impl MovePath<Right, End> for PatternPrefixPath {
    fn move_leaf<Trav: Traversable>(
        &mut self,
        location: &mut ChildLocation,
        trav: &Trav::Guard<'_>,
    ) -> ControlFlow<()> {
        location.advance_leaf(trav)
    }
}
impl_root! { PatternRoot for PatternPrefixPath, self => self.root.borrow() }
impl RootChildPos<Start> for PatternPrefixPath {
    fn root_child_pos(&self) -> usize {
        0
    }
}
impl<R: PathRole> PathChild<R> for PatternPrefixPath where Self: HasPath<R> + PatternRootChild<R> {}

impl<R> PatternRootChild<R> for PatternPrefixPath where PatternPrefixPath: RootChildPos<R> {}

impl HasPath<End> for PatternPrefixPath {
    fn path(&self) -> &Vec<ChildLocation> {
        self.role_path.path()
    }
    fn path_mut(&mut self) -> &mut Vec<ChildLocation> {
        self.role_path.path_mut()
    }
}
//impl PathChild<Start> for PatternPrefixPath {
//    fn path_child_location(&self) -> Option<ChildLocation> {
//        None
//    }
//    fn path_child<Trav: Traversable>(
//        &self,
//        trav: &Trav,
//    ) -> Option<Child> {
//        Some(self.root_child())
//    }
//}
impl_child! { RootChild for PatternPrefixPath, self, _trav => self.pattern_root_child() }

impl FoldablePath for PatternPrefixPath {
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
    fn new_directed<D: MatchDirection, P: IntoPattern>(
        query: P
    ) -> Result<Self, (ErrorReason, Self)> {
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
