use crate::{
    graph::{
        getters::vertex::VertexSet,
        vertex::{
            child::Child,
            location::child::ChildLocation,
            wide::Wide,
        },
    },
    impl_cursor_pos,
    path::{
        accessors::{
            role::{
                End,
                Start,
            },
            root::GraphRoot,
        },
        mutators::{
            adapters::IntoAdvanced,
            append::PathAppend,
            lower::PathLower,
            move_path::{
                Advance,
                Retract,
            },
        },
        structs::rooted::index_range::IndexRangePath,
        RoleChildPath,
    },
    traversal::{
        cache::key::{
            directed::{
                up::UpKey,
                DirectedKey,
            },
            prev::ToPrev,
            props::{
                CursorPosition,
                LeafKey,
                RootKey,
                TargetKey,
            },
        },
        state::{
            bottom_up::parent::ParentState,
            top_down::end::{
                EndKind,
                EndReason,
                EndState,
                RangeEnd,
            },
            BaseState,
        },
        traversable::Traversable,
    },
};
use derive_more::{
    derive::Deref,
    DerefMut,
};
use derive_new::new;
use itertools::Itertools;
use std::{
    cmp::Ordering,
    collections::VecDeque,
};

use std::fmt::Debug;

use super::super::StateNext;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PathPairMode {
    GraphMajor,
    QueryMajor,
}

impl_cursor_pos! {
    CursorPosition for ChildState, self => self.cursor.relative_pos
}
#[derive(Clone, Debug)]
pub enum ChildMatchState {
    Mismatch(EndState),
    Match(ChildState),
}
#[derive(Clone, Debug)]
pub enum TDNext {
    MatchState(ChildMatchState),
    Prefixes(StateNext<ChildBatch>),
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
#[derive(Clone, Debug, PartialEq, Eq, Deref, new)]
pub struct ChildCtx<Trav: Traversable> {
    #[deref]
    pub state: ChildState,
    pub trav: Trav,
}
impl<'a, Trav: Traversable> ChildCtx<Trav> {
    pub fn compare(self) -> ChildMatchState {
        self.into_batch_iter().find_map(|flow| flow).unwrap()
    }
    pub fn into_batch_iter(self) -> ChildBatchIterator<Trav> {
        ChildBatchIterator {
            children: FromIterator::from_iter([ChildModeCtx {
                state: self.state,
                mode: PathPairMode::GraphMajor,
            }]),
            trav: self.trav,
        }
    }
}

pub type ChildBatch = VecDeque<ChildModeCtx>;

#[derive(Debug)]
pub struct ChildBatchIterator<Trav: Traversable> {
    pub children: ChildBatch,
    pub trav: Trav,
}
impl<Trav: Traversable> Iterator for ChildBatchIterator<Trav> {
    type Item = Option<ChildMatchState>;
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(cs) = self.children.pop_front() {
            Some(match cs.state.next_match(&self.trav) {
                TDNext::Prefixes(next) => {
                    self.children.extend(next.inner);
                    None
                }
                TDNext::MatchState(state) => Some(state),
            })
        } else {
            None
        }
    }
}
impl<Trav: Traversable> ChildBatchIterator<Trav> {
    pub fn find_match(mut self) -> Option<ChildState> {
        self.find_map(|flow| match flow {
            Some(ChildMatchState::Match(state)) => Some(state),
            _ => None,
        })
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
    fn prefix_states<Trav: Traversable>(
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
                        target: DirectedKey::down(sub.child, *mctx.state.cursor.cursor_pos()),
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
            PathPairMode::GraphMajor => self.state.path.path_append(location),
            PathPairMode::QueryMajor => self.state.cursor.path_append(location),
        }
    }
    pub fn major_leaf<Trav: Traversable>(
        &self,
        trav: &Trav,
    ) -> Child {
        match self.mode {
            PathPairMode::GraphMajor => self.state.path.role_leaf_child::<End, _>(trav),
            PathPairMode::QueryMajor => self.state.cursor.role_leaf_child::<End, _>(trav),
        }
    }
}
impl ChildState {
    fn mode_prefixes<'a, Trav: Traversable>(
        &self,
        trav: &Trav,
        mode: PathPairMode,
    ) -> VecDeque<ChildModeCtx> {
        ChildModeCtx {
            state: self.clone(),
            mode,
        }
        .prefix_states(trav)
    }
    pub fn next_match<Trav: Traversable>(
        self,
        trav: &Trav,
    ) -> TDNext {
        let key = self.target_key();
        let path_leaf = self.path.role_leaf_child::<End, _>(trav);
        let query_leaf = self.cursor.role_leaf_child::<End, _>(trav);

        // compare next child
        if path_leaf == query_leaf {
            TDNext::MatchState(ChildMatchState::Match(self))
        } else if path_leaf.width() == 1 && query_leaf.width() == 1 {
            TDNext::MatchState(ChildMatchState::Mismatch(self.on_mismatch(trav)))
        } else {
            TDNext::Prefixes(StateNext {
                prev: key.to_prev(0),
                inner: match path_leaf.width.cmp(&query_leaf.width) {
                    Ordering::Equal => self
                        .mode_prefixes(trav, PathPairMode::GraphMajor)
                        .into_iter()
                        .chain(self.mode_prefixes(trav, PathPairMode::QueryMajor))
                        .collect(),
                    Ordering::Greater => self.mode_prefixes(trav, PathPairMode::GraphMajor),
                    Ordering::Less => self.mode_prefixes(trav, PathPairMode::QueryMajor),
                },
            })
        }
    }

    fn on_mismatch<'a, Trav: Traversable>(
        &self,
        trav: &Trav,
    ) -> EndState {
        //let key = self.target_key();
        let BaseState {
            cursor,
            mut path,
            mut root_pos,
            prev_pos,
        } = self.base.clone();
        //cursor.retract(trav);
        //path.retract(trav);
        if let Some(index) = loop {
            if path.role_root_child_pos::<Start>() == path.role_root_child_pos::<End>() {
                if (&mut root_pos, &mut path).path_lower(trav).is_break() {
                    let graph = trav.graph();
                    let pattern = graph.expect_pattern_at(path.root.location);
                    let entry = path.start.sub_path.root_entry;
                    root_pos = prev_pos;
                    break Some(pattern[entry]);
                }
            } else {
                break None;
            }
        } {
            EndState {
                root_pos,
                cursor: cursor,
                reason: EndReason::Mismatch,
                kind: EndKind::Complete(index),
            }
        } else {
            EndState {
                root_pos,
                reason: EndReason::Mismatch,
                kind: RangeEnd {
                    target: DirectedKey::down(
                        path.role_leaf_child::<End, _>(trav),
                        cursor.relative_pos,
                    ),
                    path,
                }
                .simplify_to_end(trav),
                cursor: cursor.clone(),
            }
        }
    }
}

impl IntoAdvanced for ChildState {
    type Next = Self;
    fn into_advanced<Trav: Traversable>(
        mut self,
        trav: &Trav,
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

impl Ord for Child {
    fn cmp(
        &self,
        other: &Self,
    ) -> Ordering {
        self.width().cmp(&other.width())
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
        self.target
    }
}
