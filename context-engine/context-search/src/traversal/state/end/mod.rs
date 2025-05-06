use context_trace::{
    graph::vertex::{
        child::Child,
        location::child::ChildLocation,
    },
    impl_cursor_pos,
    impl_root,
    path::{
        accessors::{
            child::root::GraphRootChild,
            complete::PathComplete,
            role::Start,
            root::GraphRoot,
        },
        mutators::{
            move_path::key::TokenPosition,
            simplify::PathSimplify,
        },
        structs::rooted::{
            role_path::IndexStartPath,
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
                },
                props::{
                    CursorPosition,
                    RootKey,
                    TargetKey,
                },
            },
            //new::NewEnd,
        },
        has_graph::HasGraph,
        traceable::Traceable,
        StateDirection,
        TraceContext,
    },
};
use context_trace::{
    path::{
        accessors::{
            has_path::HasRootedRolePath,
            role::End,
        },
        structs::rooted::index_range::IndexRangePath,
    },
    trace::cache::key::directed::down::DownKey,
};
use postfix::PostfixEnd;
use prefix::PrefixEnd;
use range::RangeEnd;

use super::{
    cursor::PatternPrefixCursor,
    parent::ParentState,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EndKind {
    Range(RangeEnd),
    Postfix(PostfixEnd),
    Prefix(PrefixEnd),
    Complete(Child),
}
impl EndKind {
    pub fn from_range_path<G: HasGraph>(
        mut path: IndexRangePath,
        root_pos: TokenPosition,
        target: DownKey,
        trav: &G,
    ) -> Self {
        path.child_path_mut::<Start>().simplify(trav);
        path.child_path_mut::<End>().simplify(trav);

        match (
            path.is_at_border::<_, Start>(trav.graph()),
            path.raw_child_path::<Start>().is_empty(),
            path.is_at_border::<_, End>(trav.graph()),
            path.raw_child_path::<End>().is_empty(),
        ) {
            (true, true, true, true) => EndKind::Complete(path.root_parent()),
            (true, true, false, _) | (true, true, true, false) =>
                EndKind::Prefix(PrefixEnd {
                    path: path.into(),
                    target,
                }),
            (false, _, true, true) | (true, false, true, true) => {
                let path: IndexStartPath = path.into();
                EndKind::Postfix(PostfixEnd { path, root_pos })
            },
            _ => EndKind::Range(RangeEnd {
                path,
                root_pos,
                target,
            }),
        }
    }
    pub fn from_start_path<G: HasGraph>(
        mut path: IndexStartPath,
        root_pos: TokenPosition,
        trav: &G,
    ) -> Self {
        path.role_path.simplify(trav);
        match (
            path.is_at_border::<_, Start>(trav.graph()),
            path.role_path.raw_child_path().is_empty(),
        ) {
            (true, true) => EndKind::Complete(path.root_parent()),
            _ => {
                EndKind::Postfix(PostfixEnd {
                    path,
                    //inner_width: path.get_post_width(trav),
                    root_pos,
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
pub mod postfix;
pub mod prefix;
pub mod range;
// End types:
// - top down match-mismatch
// - top down match-query end
// - bottom up-no matching parents

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EndState {
    pub reason: EndReason,
    pub kind: EndKind,
    pub cursor: PatternPrefixCursor,
}
impl_cursor_pos! {
    CursorPosition for EndState, self => self.cursor.relative_pos
}

impl Traceable for &EndState {
    fn trace<G: HasGraph>(
        self,
        ctx: &mut TraceContext<G>,
    ) {
        match &self.kind {
            EndKind::Range(p) => p.trace(ctx),
            EndKind::Prefix(p) => p.trace(ctx),
            EndKind::Postfix(p) => p.trace(ctx),
            _ => {},
        }
    }
}
#[derive(Clone, Debug)]
pub struct TraceStart<'a>(pub &'a EndState, pub usize);

impl<'a> Traceable for TraceStart<'a> {
    fn trace<G: HasGraph>(
        self,
        ctx: &mut TraceContext<G>,
    ) {
        match &self.0.kind {
            EndKind::Postfix(p) => Some(p.clone()),
            EndKind::Range(p) => Some(PostfixEnd {
                path: p.path.rooted_role_path(),
                root_pos: p.root_pos,
            }),
            _ => None,
        }
        .map(|mut p| {
            p.path.role_path.sub_path.path.drain(0..self.1);
            p.trace(ctx)
        });
    }
}
impl EndState {
    pub fn with_reason<G: HasGraph>(
        trav: G,
        reason: EndReason,
        parent: ParentState,
    ) -> Self {
        Self {
            reason,
            kind: EndKind::from_start_path(parent.path, parent.root_pos, &trav),
            cursor: parent.cursor,
        }
    }
    pub fn query_end<G: HasGraph>(
        trav: G,
        parent: ParentState,
    ) -> Self {
        Self::with_reason(trav, EndReason::QueryEnd, parent)
    }
    pub fn mismatch<G: HasGraph>(
        trav: G,
        parent: ParentState,
    ) -> Self {
        Self::with_reason(trav, EndReason::Mismatch, parent)
    }
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
    pub fn start_len(&self) -> usize {
        self.start_path()
            .map(|p| p.sub_path.len())
            .unwrap_or_default()
    }
    pub fn start_path(&self) -> Option<RootedSplitPathRef<'_>> {
        match &self.kind {
            EndKind::Range(e) => Some(e.path.start_path()),
            EndKind::Postfix(e) => Some((&e.path).into()),
            EndKind::Prefix(_) => None,
            EndKind::Complete(_) => None,
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

impl TargetKey for EndState {
    fn target_key(&self) -> DirectedKey {
        match &self.kind {
            EndKind::Range(p) => p.target.clone().into(),
            EndKind::Postfix(_) => self.root_key().into(),
            EndKind::Prefix(p) => p.target.clone().into(),
            EndKind::Complete(c) => DirectedKey::up(*c, *self.cursor_pos()),
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
            match &self.kind {
                EndKind::Range(s) => s.root_pos.into(),
                EndKind::Postfix(p) => p.root_pos.into(),
                EndKind::Prefix(_) => 0.into(),
                EndKind::Complete(_) => 0.into(),
            },
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
