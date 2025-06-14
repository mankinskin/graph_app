use crate::traversal::state::{
    cursor::PatternCursor,
    end::{
        EndKind,
        EndReason,
        EndState,
    },
    ChildMatchState::{
        self,
        Match,
        Mismatch,
    },
};
use context_trace::{
    graph::vertex::wide::Wide,
    path::{
        accessors::role::{
            End,
            Start,
        },
        mutators::{
            adapters::IntoAdvanced,
            lower::PathLower,
        },
        RolePathUtils,
    },
    trace::{
        cache::key::{
            directed::down::DownKey,
            props::CursorPosition,
        },
        child::{
            iterator::ChildQueue,
            state::{
                ChildState,
                PrefixStates,
            },
        },
        has_graph::HasGraph,
        state::BaseState,
    },
};
use derive_more::{
    Deref,
    DerefMut,
};
use std::{
    cmp::Ordering,
    collections::VecDeque,
    fmt::Debug,
};
use tracing::debug;
use CompareNext::*;
use PathPairMode::*;

use super::parent::ParentCompareState;

pub type CompareQueue = VecDeque<CompareState>;

#[derive(Clone, Debug, PartialEq, Eq, Copy)]
pub enum PathPairMode {
    GraphMajor,
    QueryMajor,
}

#[derive(Clone, Debug, PartialEq, Eq, Deref, DerefMut)]
pub struct CompareState {
    #[deref]
    #[deref_mut]
    pub child_state: ChildState,
    pub cursor: PatternCursor,
    pub target: DownKey,
    pub mode: PathPairMode,
}

#[derive(Clone, Debug)]
pub enum CompareNext {
    MatchState(ChildMatchState),
    Prefixes(ChildQueue<CompareState>),
}
impl CompareState {
    fn mode_prefixes<'a, G: HasGraph>(
        &self,
        trav: &G,
        mode: PathPairMode,
    ) -> VecDeque<Self> {
        Self {
            mode,
            ..self.clone()
        }
        .prefix_states(trav)
    }
    pub fn parent_state(&self) -> ParentCompareState {
        ParentCompareState {
            parent_state: self.child_state.parent_state(),
            cursor: self.cursor.clone(),
        }
    }
    /// generate child states for index prefixes
    pub fn prefix_states<G: HasGraph>(
        &self,
        trav: &G,
    ) -> VecDeque<Self> {
        match self.mode {
            GraphMajor => self
                .child_state
                .prefix_states(trav)
                .into_iter()
                .map(|(sub, child_state)| Self {
                    target: DownKey::new(
                        sub.child,
                        (*self.cursor.cursor_pos()).into(),
                    ),
                    child_state,
                    mode: self.mode,
                    cursor: self.cursor.clone(),
                })
                .collect(),
            QueryMajor => self
                .cursor
                .prefix_states(trav)
                .into_iter()
                .map(|(sub, cursor)| Self {
                    target: DownKey::new(
                        sub.child,
                        (*cursor.cursor_pos()).into(),
                    ),
                    child_state: self.child_state.clone(),
                    mode: self.mode,
                    cursor,
                })
                .collect(),
        }
    }
    pub fn next_match<G: HasGraph>(
        self,
        trav: &G,
    ) -> CompareNext {
        use Ordering::*;
        let path_leaf = self.path.role_leaf_child::<End, _>(trav);
        let query_leaf = self.cursor.role_leaf_child::<End, _>(trav);

        if path_leaf == query_leaf {
            debug!(
                "Matched\n\tlabel: {}\n\troot: {}\n\tpos: {}",
                trav.graph().index_string(path_leaf),
                trav.graph().index_string(self.path.root.location.parent),
                self.cursor.width()
            );
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
        let CompareState {
            child_state:
                ChildState {
                    base:
                        BaseState {
                            mut path,
                            mut root_pos,
                            prev_pos,
                        },
                    ..
                },
            cursor,
            ..
        } = self;
        // TODO: Fix this
        let index = loop {
            if path.role_root_child_index::<Start>()
                == path.role_root_child_index::<End>()
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
            let target = DownKey::new(
                path.role_leaf_child::<End, _>(trav),
                cursor.relative_pos.into(),
            );
            EndKind::from_range_path(path, root_pos, target, trav)
        };
        EndState {
            reason: Mismatch,
            kind,
            cursor,
        }
    }
}

impl Into<ChildQueue<CompareState>> for CompareState {
    fn into(self) -> ChildQueue<Self> {
        VecDeque::from_iter([self])
    }
}

impl IntoAdvanced for CompareState {
    type Next = Self;
    fn into_advanced<G: HasGraph>(
        self,
        trav: &G,
    ) -> Result<Self, Self> {
        match self.child_state.into_advanced(trav) {
            Ok(child_state) => Ok(Self {
                child_state,
                ..self
            }),
            Err(child_state) => Ok(Self {
                child_state,
                ..self
            }),
        }
    }
}
