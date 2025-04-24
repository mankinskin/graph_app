use crate::traversal::state::cursor::PatternRangeCursor;
use context_trace::{
    graph::vertex::{
        child::Child,
        location::child::ChildLocation,
        pattern::pattern_width,
        wide::Wide,
    },
    impl_cursor_pos,
    impl_root,
    path::{
        accessors::{
            border::PathBorder,
            child::root::GraphRootChild,
            complete::PathComplete,
            role::{
                End,
                Start,
            },
            root::GraphRoot,
        },
        mutators::{
            move_path::key::TokenPosition,
            simplify::PathSimplify,
        },
        structs::rooted::{
            index_range::IndexRangePath,
            role_path::{
                IndexEndPath,
                IndexRolePath,
                IndexStartPath,
            },
            split_path::RootedSplitPathRef,
        },
        GetRoleChildPath,
    },
    trace::{
        cache::{
            key::{
                directed::{
                    up::UpKey,
                    DirectedKey,
                    HasTokenPosition,
                },
                props::{
                    CursorPosition,
                    LeafKey,
                    RootKey,
                    TargetKey,
                },
            },
            new::{
                NewEnd,
                NewRangeEnd,
            },
        },
        has_graph::HasGraph,
        StateDirection,
    },
};
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EndKind {
    Range(RangeEnd),
    Postfix(PostfixEnd),
    Prefix(PrefixEnd),
    Complete(Child),
}
impl EndKind {
    pub fn simplify_path<G: HasGraph>(
        mut path: IndexRolePath<Start>,
        trav: &G,
    ) -> Self {
        path.role_path.simplify(trav);
        match (
            Start::is_at_border(
                trav.graph(),
                path.role_root_child_location::<Start>(),
            ),
            path.role_path.raw_child_path::<Start>().is_empty(),
        ) {
            (true, true) => EndKind::Complete(path.root_parent()),
            _ => {
                let graph = trav.graph();
                let root = path.role_root_child_location();
                let pattern = graph.expect_pattern_at(root.clone());
                EndKind::Postfix(PostfixEnd {
                    path,
                    inner_width: pattern_width(&pattern[root.sub_index + 1..]),
                })
            },
        }
    }
}
impl PathComplete for EndKind {
    fn as_complete(&self) -> Option<Child> {
        match self {
            Self::Complete(c) => Some(*c),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum EndReason {
    QueryEnd,
    Mismatch,
}

// End types:
// - top down match-mismatch
// - top down match-query end
// - bottom up-no matching parents
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RangeEnd {
    pub path: IndexRangePath,
    pub target: DirectedKey,
}
impl LeafKey for RangeEnd {
    fn leaf_location(&self) -> ChildLocation {
        self.path.leaf_location()
    }
}
impl From<&RangeEnd> for NewRangeEnd {
    fn from(state: &RangeEnd) -> Self {
        Self {
            target: state.target.clone(),
            entry: GraphRootChild::<Start>::root_child_location(&state.path),
        }
    }
}

impl RangeEnd {
    pub fn simplify_to_end<G: HasGraph>(
        mut self,
        trav: &G,
    ) -> EndKind {
        self.path.child_path_mut::<Start>().simplify(trav);
        self.path.child_path_mut::<End>().simplify(trav);

        match (
            Start::is_at_border(
                trav.graph(),
                self.path.role_root_child_location::<Start>(),
            ),
            self.path.raw_child_path::<Start>().is_empty(),
            End::is_at_border(
                trav.graph(),
                self.path.role_root_child_location::<End>(),
            ),
            self.path.raw_child_path::<End>().is_empty(),
        ) {
            (true, true, true, true) =>
                EndKind::Complete(self.path.root_parent()),
            (true, true, false, _) | (true, true, true, false) =>
                EndKind::Prefix(PrefixEnd {
                    path: self.path.into(),
                    target: self.target,
                }),
            (false, _, true, true) | (true, false, true, true) => {
                let graph = trav.graph();
                let path: IndexStartPath = self.path.into();
                let root = path.role_root_child_location();
                let pattern = graph.expect_pattern_at(root.clone());
                EndKind::Postfix(PostfixEnd {
                    path,
                    inner_width: pattern_width(&pattern[root.sub_index + 1..]),
                })
            },
            _ => EndKind::Range(self),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PrefixEnd {
    pub path: IndexEndPath,
    pub target: DirectedKey,
}
use context_trace::trace::{
    traceable::Traceable,
    TraceContext,
};
impl Traceable for &PrefixEnd {
    fn trace<G: HasGraph>(
        &self,
        ctx: &mut TraceContext<G>,
    ) {
        ctx.trace_prefix_path(&self.path, true)
    }
}
impl Traceable for &RangeEnd {
    fn trace<G: HasGraph>(
        &self,
        ctx: &mut TraceContext<G>,
    ) {
        let root = self.path.role_root_child_location::<Start>().parent;
        ctx.trace_range_path(
            &self.path,
            UpKey {
                index: root,
                pos: (*self.target.pos.pos()).into(),
            },
            true,
        )
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PostfixEnd {
    pub path: IndexStartPath,
    pub inner_width: usize,
}
impl Traceable for &PostfixEnd {
    fn trace<G: HasGraph>(
        &self,
        ctx: &mut TraceContext<G>,
    ) {
        ctx.trace_postfix_path(
            &self.path,
            UpKey::new(self.path.root.location.parent.into(), 0.into()).into(),
            true,
        )
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EndState {
    pub reason: EndReason,
    pub root_pos: TokenPosition,
    pub kind: EndKind,
    pub cursor: PatternRangeCursor,
}
impl_cursor_pos! {
    CursorPosition for EndState, self => self.cursor.relative_pos
}

impl Traceable for EndState {
    fn trace<G: HasGraph>(
        &self,
        ctx: &mut TraceContext<G>,
    ) {
        match &self.kind {
            EndKind::Range(p) => p.trace(ctx),
            EndKind::Prefix(p) => p.trace(ctx),
            _ => {},
        }
    }
}
impl EndState {
    pub fn is_final(&self) -> bool {
        self.reason == EndReason::QueryEnd
            && matches!(self.kind, EndKind::Complete(_))
    }
    pub fn entry_location(&self) -> Option<ChildLocation> {
        match &self.kind {
            EndKind::Range(state) =>
                Some(GraphRootChild::<Start>::root_child_location(&state.path)),
            EndKind::Postfix(_) => None,
            EndKind::Prefix(_) => None,
            EndKind::Complete(_) => None,
        }
    }
    pub fn state_direction(&self) -> StateDirection {
        match self.kind {
            EndKind::Range(_) => StateDirection::TopDown,
            EndKind::Postfix(_) => StateDirection::BottomUp,
            EndKind::Prefix(_) => StateDirection::TopDown,
            EndKind::Complete(_) => StateDirection::BottomUp,
        }
    }
    pub fn end_path(&self) -> Option<RootedSplitPathRef<'_>> {
        match &self.kind {
            EndKind::Range(e) => Some(e.path.end_path()),
            EndKind::Postfix(_) => None,
            EndKind::Prefix(e) => Some((&e.path).into()),
            EndKind::Complete(_) => None,
        }
    }
    pub fn is_complete(&self) -> bool {
        matches!(self.kind, EndKind::Complete(_))
    }
}

impl From<&EndState> for NewEnd {
    fn from(state: &EndState) -> Self {
        match &state.kind {
            EndKind::Range(range) => Self::Range(range.into()),
            EndKind::Postfix(_) => Self::Postfix(state.root_key()),
            EndKind::Prefix(_) => Self::Prefix(state.target_key()),
            EndKind::Complete(_) => Self::Complete(state.target_key()),
        }
    }
}

impl TargetKey for EndState {
    fn target_key(&self) -> DirectedKey {
        match &self.kind {
            EndKind::Range(p) => p.target.clone(),
            EndKind::Postfix(_) => self.root_key().into(),
            EndKind::Prefix(p) => p.target.clone(),
            EndKind::Complete(c) => DirectedKey::up(*c, *self.cursor_pos()),
        }
    }
}

impl Wide for EndState {
    fn width(&self) -> usize {
        match &self.kind {
            EndKind::Range(p) => p.target.pos.pos().0 + p.target.index.width(),
            EndKind::Prefix(p) => p.target.pos.pos().0 + p.target.index.width(),
            EndKind::Postfix(p) => self.root_pos.0 + p.inner_width,
            EndKind::Complete(c) => c.width(),
        }
    }
}
impl RootKey for EndState {
    fn root_key(&self) -> UpKey {
        UpKey::new(
            match &self.kind {
                EndKind::Range(s) => s.path.root_parent(),
                EndKind::Postfix(p) => p.path.root_parent(),
                EndKind::Prefix(p) => p.path.root_parent(),
                EndKind::Complete(c) => *c,
            },
            self.root_pos.into(),
        )
    }
}
impl_root! { GraphRoot for EndState, self =>
    match &self.kind {
        EndKind::Complete(c) => *c,
        EndKind::Range(p) => p.path.root_parent(),
        EndKind::Postfix(p) => p.path.root_parent(),
        EndKind::Prefix(p) => p.path.root_parent(),
    }
}
