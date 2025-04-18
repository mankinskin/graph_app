use crate::traversal::state::{
    child::ChildState,
    end::EndState,
};
use context_trace::{
    graph::{
        getters::vertex::VertexSet,
        vertex::{
            child::Child,
            location::child::ChildLocation,
        },
    },
    path::{
        accessors::role::End,
        mutators::append::PathAppend,
        RoleChildPath,
    },
    trace::{
        cache::key::{
            directed::DirectedKey,
            props::CursorPosition,
        },
        traversable::Traversable,
    },
};
use derive_more::{
    derive::Deref,
    DerefMut,
};
use itertools::Itertools;
use std::{
    collections::VecDeque,
    fmt::Debug,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PathPairMode {
    GraphMajor,
    QueryMajor,
}
use PathPairMode::*;
#[derive(Clone, Debug)]
pub enum ChildMatchState {
    Mismatch(EndState),
    Match(ChildState),
}
use ChildMatchState::*;

#[derive(Clone, Debug)]
pub enum TDNext {
    MatchState(ChildMatchState),
    Prefixes(ChildQueue),
}
use TDNext::*;

pub type ChildQueue = VecDeque<ChildModeCtx>;

#[derive(Debug)]
pub struct ChildIterator<Trav: Traversable> {
    pub children: ChildQueue,
    pub trav: Trav,
}
impl<Trav: Traversable> ChildIterator<Trav> {
    pub fn new(
        trav: Trav,
        state: ChildState,
    ) -> Self {
        Self {
            children: FromIterator::from_iter([ChildModeCtx {
                state,
                mode: GraphMajor,
            }]),
            trav,
        }
    }
    pub fn find_match(self) -> Option<ChildState> {
        match self.compare() {
            Mismatch(_) => None,
            Match(state) => Some(state),
        }
    }
    pub fn compare(mut self) -> ChildMatchState {
        self.find_map(|flow| flow).unwrap()
    }
}

impl<Trav: Traversable> Iterator for ChildIterator<Trav> {
    type Item = Option<ChildMatchState>;
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(cs) = self.children.pop_front() {
            Some(match cs.state.next_match(&self.trav) {
                Prefixes(next) => {
                    self.children.extend(next);
                    None
                },
                MatchState(state) => Some(state),
            })
        } else {
            None
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Deref, DerefMut)]
pub struct ChildModeCtx {
    #[deref]
    #[deref_mut]
    pub state: ChildState,
    pub mode: PathPairMode,
}

impl ChildModeCtx {
    /// generate child states for index prefixes
    pub fn prefix_states<Trav: Traversable>(
        &self,
        trav: &Trav,
    ) -> VecDeque<ChildModeCtx> {
        let leaf = self.major_leaf(&trav);
        trav.graph()
            .expect_vertex(leaf)
            .prefix_children::<Trav>()
            .iter()
            .sorted_unstable_by(|a, b| b.child.width.cmp(&a.child.width))
            .map(|sub| {
                let mut mctx = self.clone();
                mctx.push_major(leaf.to_child_location(sub.location));
                ChildModeCtx {
                    state: ChildState {
                        target: DirectedKey::down(
                            sub.child,
                            *mctx.state.cursor.cursor_pos(),
                        ),
                        ..mctx.state
                    },
                    mode: mctx.mode,
                }
            })
            .collect()
    }
    pub fn push_major(
        &mut self,
        location: ChildLocation,
    ) {
        match self.mode {
            GraphMajor => self.state.path.path_append(location),
            QueryMajor => self.state.cursor.path_append(location),
        }
    }
    pub fn major_leaf<Trav: Traversable>(
        &self,
        trav: &Trav,
    ) -> Child {
        match self.mode {
            GraphMajor => self.state.path.role_leaf_child::<End, _>(trav),
            QueryMajor => self.state.cursor.role_leaf_child::<End, _>(trav),
        }
    }
}
