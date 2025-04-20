pub mod batch;
use crate::traversal::state::{
    child::{
        batch::ChildMatchState::{
            Match,
            Mismatch,
        },
        PathPairMode::{
            GraphMajor,
            QueryMajor,
        },
        TDNext::{
            MatchState,
            Prefixes,
        },
    },
    end::{
        EndKind,
        EndReason,
        EndState,
        RangeEnd,
    },
    parent::ParentState,
    BaseState,
};
use batch::{
    ChildModeCtx,
    PathPairMode,
    TDNext,
};
use context_trace::{
    graph::vertex::{
        location::child::ChildLocation,
        wide::Wide,
    },
    impl_cursor_pos,
    path::{
        accessors::{
            has_path::HasRootedRolePath,
            role::{
                End,
                Start,
            },
            root::GraphRoot,
        },
        mutators::{
            adapters::IntoAdvanced,
            lower::PathLower,
            move_path::advance::Advance,
        },
        structs::rooted::index_range::IndexRangePath,
        RoleChildPath,
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
                    LeafKey,
                    RootKey,
                    TargetKey,
                },
            },
            new::NewChild,
        },
        has_graph::HasGraph,
    },
};
use derive_more::{
    derive::Deref,
    DerefMut,
};
use std::{
    cmp::Ordering,
    collections::VecDeque,
    fmt::Debug,
};
impl_cursor_pos! {
    CursorPosition for ChildState, self => self.cursor.relative_pos
}

#[derive(Clone, Debug, PartialEq, Eq, Deref, DerefMut)]
pub struct RootChildState {
    #[deref]
    #[deref_mut]
    pub child: ChildState,
    pub root_parent: ParentState,
}

#[derive(Clone, Debug, PartialEq, Eq, Deref, DerefMut)]
pub struct ChildState {
    #[deref]
    #[deref_mut]
    pub base: BaseState<IndexRangePath>,
    pub target: DirectedKey,
}

impl ChildState {
    pub fn root_parent(&self) -> ParentState {
        ParentState {
            path: self.base.path.rooted_role_path(),
            ..self.base.clone()
        }
    }
    fn mode_prefixes<'a, G: HasGraph>(
        &self,
        trav: &G,
        mode: PathPairMode,
    ) -> VecDeque<ChildModeCtx> {
        ChildModeCtx {
            state: self.clone(),
            mode,
        }
        .prefix_states(trav)
    }
    pub fn next_match<G: HasGraph>(
        self,
        trav: &G,
    ) -> TDNext {
        use Ordering::*;
        let path_leaf = self.path.role_leaf_child::<End, _>(trav);
        let query_leaf = self.cursor.role_leaf_child::<End, _>(trav);

        if path_leaf == query_leaf {
            MatchState(Match(self))
        } else if path_leaf.width() == 1 && query_leaf.width() == 1 {
            MatchState(Mismatch(self.on_mismatch(trav)))
        } else {
            Prefixes(match path_leaf.width.cmp(&query_leaf.width) {
                Equal => self
                    .mode_prefixes(trav, GraphMajor)
                    .into_iter()
                    .chain(self.mode_prefixes(trav, QueryMajor))
                    .collect(),
                Greater => self.mode_prefixes(trav, GraphMajor),
                Less => self.mode_prefixes(trav, QueryMajor),
            })
        }
    }

    fn on_mismatch<'a, G: HasGraph>(
        self,
        trav: &G,
    ) -> EndState {
        use EndKind::*;
        use EndReason::*;
        let BaseState {
            cursor,
            mut path,
            mut root_pos,
            prev_pos,
        } = self.base;
        // TODO: Fix this
        let index = loop {
            if path.role_root_child_pos::<Start>()
                == path.role_root_child_pos::<End>()
            {
                if (&mut root_pos, &mut path).path_lower(trav).is_break() {
                    let graph = trav.graph();
                    let pattern =
                        graph.expect_pattern_at(path.clone().root.location);
                    let entry = path.start.sub_path.root_entry;
                    root_pos = prev_pos;
                    break Some(pattern[entry]);
                }
            } else {
                break None;
            }
        };
        let kind = if let Some(index) = index {
            Complete(index)
        } else {
            RangeEnd {
                target: DirectedKey::down(
                    path.role_leaf_child::<End, _>(trav),
                    cursor.relative_pos,
                ),
                path,
            }
            .simplify_to_end(trav)
        };
        EndState {
            reason: Mismatch,
            root_pos,
            kind,
            cursor,
        }
    }
}

impl From<ChildState> for NewChild {
    fn from(state: ChildState) -> Self {
        Self {
            root: state.root_key(),
            target: state.target_key(),
            end_leaf: state.path.role_leaf_child_location::<End>(),
        }
    }
}

impl IntoAdvanced for ChildState {
    type Next = Self;
    fn into_advanced<G: HasGraph>(
        mut self,
        trav: &G,
    ) -> Result<Self, Self> {
        if self.base.path.advance(trav).is_continue() {
            // gen next child
            Ok(Self {
                target: DirectedKey::down(
                    self.base.path.role_leaf_child::<End, _>(&trav),
                    *self.cursor_pos(),
                ),
                ..self
            })
        } else {
            Err(self)
        }
    }
}

impl Ord for ChildState {
    fn cmp(
        &self,
        other: &Self,
    ) -> Ordering {
        self.path.root_parent().cmp(&other.path.root_parent())
    }
}

impl PartialOrd for ChildState {
    fn partial_cmp(
        &self,
        other: &Self,
    ) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl RootKey for ChildState {
    fn root_key(&self) -> UpKey {
        UpKey::new(self.path.root_parent(), self.root_pos.into())
    }
}

impl LeafKey for ChildState {
    fn leaf_location(&self) -> ChildLocation {
        self.path.leaf_location()
    }
}
impl TargetKey for ChildState {
    fn target_key(&self) -> DirectedKey {
        self.target.clone()
    }
}
