use super::parent::ParentState;
use crate::traversal::{
    state::parent::IntoPrimer,
    ControlFlow,
};
use context_trace::{
    direction::{
        pattern::PatternDirection,
        Left,
        Right,
    },
    graph::vertex::{
        child::Child,
        location::{
            child::ChildLocation,
            pattern::IntoPatternLocation,
        },
        wide::Wide,
    },
    impl_cursor_pos,
    path::{
        accessors::{
            child::{
                root::RootChild,
                PathChild,
                RootChildPos,
                RootChildPosMut,
            },
            has_path::HasPath,
            role::{
                End,
                PathRole,
            },
        },
        mutators::{
            append::PathAppend,
            move_path::{
                key::{
                    AdvanceKey,
                    MoveKey,
                    RetractKey,
                    TokenPosition,
                },
                leaf::{
                    AdvanceLeaf,
                    KeyedLeaf,
                    RetractLeaf,
                },
                path::MovePath,
                root::MoveRootPos,
            },
            pop::PathPop,
        },
        structs::{
            query_range_path::FoldablePath,
            role_path::RolePath,
            rooted::{
                pattern_range::PatternRangePath,
                role_path::RootedRolePath,
                root::IndexRoot,
            },
            sub_path::SubPath,
        },
    },
    trace::traversable::{
        TravDir,
        Traversable,
    },
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PathCursor<P: FoldablePath> {
    pub path: P,
    /// position relative to start of path
    pub relative_pos: TokenPosition,
}
impl<P: FoldablePath> PathPop for PathCursor<P> {
    fn path_pop(&mut self) -> Option<ChildLocation> {
        self.path.path_pop()
    }
}
impl<P: FoldablePath> PathAppend for PathCursor<P> {
    fn path_append(
        &mut self,
        parent_entry: ChildLocation,
    ) {
        self.path.path_append(parent_entry);
    }
}
impl<R: PathRole, P: RootChild<R> + FoldablePath> RootChild<R>
    for PathCursor<P>
{
    fn root_child<Trav: Traversable>(
        &self,
        trav: &Trav,
    ) -> Child {
        self.path.root_child(trav)
    }
}
impl<R: PathRole, P: FoldablePath + HasPath<R>> HasPath<R> for PathCursor<P> {
    fn path(&self) -> &Vec<ChildLocation> {
        self.path.path()
    }
    fn path_mut(&mut self) -> &mut Vec<ChildLocation> {
        self.path.path_mut()
    }
}

impl<R: PathRole, P: FoldablePath + PathChild<R>> PathChild<R>
    for PathCursor<P>
{
    fn path_child_location(&self) -> Option<ChildLocation> {
        self.path.path_child_location()
    }
    fn path_child<Trav: Traversable>(
        &self,
        trav: &Trav,
    ) -> Option<Child> {
        self.path.path_child(trav)
    }
}
impl<R: PathRole, P: RootChildPos<R> + FoldablePath> RootChildPos<R>
    for PathCursor<P>
{
    fn root_child_pos(&self) -> usize {
        RootChildPos::<R>::root_child_pos(&self.path)
    }
}

pub type PatternRangeCursor = PathCursor<PatternRangePath>;
impl_cursor_pos! {
    CursorPosition for PatternRangeCursor, self => self.relative_pos
}

impl MovePath<Right, End> for PatternRangeCursor {
    fn move_leaf<Trav: Traversable>(
        &mut self,
        location: &mut ChildLocation,
        trav: &Trav::Guard<'_>,
    ) -> ControlFlow<()> {
        KeyedLeaf::new(self, location).advance_leaf(trav)
    }
}
impl MovePath<Left, End> for PatternRangeCursor {
    fn move_leaf<Trav: Traversable>(
        &mut self,
        location: &mut ChildLocation,
        trav: &Trav::Guard<'_>,
    ) -> ControlFlow<()> {
        KeyedLeaf::new(self, location).retract_leaf(trav)
    }
}

impl MoveRootPos<Right, End> for PatternRangeCursor {
    fn move_root_pos<Trav: Traversable>(
        &mut self,
        _trav: &Trav,
    ) -> ControlFlow<()> {
        let pattern = &self.path.root;
        if let Some(next) = TravDir::<Trav>::pattern_index_next(
            pattern,
            self.path.end.root_child_pos(),
        ) {
            self.advance_key(pattern[self.path.end.root_child_pos()].width());
            *self.path.end.root_child_pos_mut() = next;
            ControlFlow::Continue(())
        } else {
            ControlFlow::Break(())
        }
    }
}
impl MoveRootPos<Left, End> for PatternRangeCursor {
    fn move_root_pos<Trav: Traversable>(
        &mut self,
        _trav: &Trav,
    ) -> ControlFlow<()> {
        let pattern = &self.path.root;
        if let Some(prev) = TravDir::<Trav>::pattern_index_prev(
            pattern,
            self.path.end.root_child_pos(),
        ) {
            self.retract_key(pattern[self.path.end.root_child_pos()].width());
            *self.path.end.root_child_pos_mut() = prev;
            ControlFlow::Continue(())
        } else {
            ControlFlow::Break(())
        }
    }
}
impl MoveKey<Right> for PatternRangeCursor {
    type Delta = usize;
    fn move_key(
        &mut self,
        delta: Self::Delta,
    ) {
        self.relative_pos.advance_key(delta)
    }
}
impl MoveKey<Left> for PatternRangeCursor {
    type Delta = usize;
    fn move_key(
        &mut self,
        delta: Self::Delta,
    ) {
        self.relative_pos.retract_key(delta)
    }
}

impl IntoPrimer for (Child, PatternRangeCursor) {
    fn into_primer<Trav: Traversable>(
        self,
        _trav: &Trav,
        parent_entry: ChildLocation,
    ) -> ParentState {
        let (c, cursor) = self;
        let width = c.width().into();
        ParentState {
            prev_pos: width,
            root_pos: width,
            path: RootedRolePath {
                root: IndexRoot {
                    location: parent_entry.into_pattern_location(),
                },
                role_path: RolePath {
                    sub_path: SubPath {
                        root_entry: parent_entry.sub_index,
                        path: vec![],
                    },
                    _ty: Default::default(),
                },
            },
            cursor,
        }
    }
}
pub trait ToCursor: FoldablePath {
    fn to_cursor(self) -> PathCursor<Self>;
}
impl<P: FoldablePath> ToCursor for P {
    fn to_cursor(self) -> PathCursor<Self> {
        PathCursor {
            path: self,
            relative_pos: TokenPosition::default(),
        }
    }
}
