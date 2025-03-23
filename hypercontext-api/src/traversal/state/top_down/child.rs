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
        TraversalContext,
        TraversalKind,
    },
};
use derive_more::{
    derive::Deref,
    DerefMut,
};
use itertools::Itertools;
use std::cmp::Ordering;
use tap::Tap;

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
pub enum MatchedNext {
    NextChild(ChildState),
    MatchedParent(ChildState),
}
#[derive(Clone, Debug)]
pub enum TDNext {
    Matched(MatchedNext),
    Mismatched(StateNext<EndState>),
    Prefixes(StateNext<Vec<ChildState>>),
}

#[derive(Clone, Debug, PartialEq, Eq, Deref, DerefMut)]
pub struct ChildState {
    #[deref]
    #[deref_mut]
    pub base: BaseState<IndexRangePath>,
    pub root_parent: ParentState,
    //pub root_prev: PrevKey,
    pub target: DirectedKey,
    pub mode: PathPairMode,
}

impl ChildState {
    pub fn major_leaf<Trav: Traversable>(
        &self,
        trav: &Trav,
    ) -> Child {
        match self.mode {
            PathPairMode::GraphMajor => self.path.role_leaf_child::<End, _>(trav),
            PathPairMode::QueryMajor => self.cursor.role_leaf_child::<End, _>(trav),
        }
    }
    pub fn push_major(
        &mut self,
        location: ChildLocation,
    ) {
        match self.mode {
            PathPairMode::GraphMajor => self.path.path_append(location),
            PathPairMode::QueryMajor => self.cursor.path_append(location),
        }
    }
    pub fn child_next_states<K: TraversalKind>(
        self,
        ctx: &mut TraversalContext<K>,
    ) -> TDNext {
        let key = self.target_key();
        let path_leaf = self.path.role_leaf_child::<End, _>(&ctx.trav);
        let query_leaf = self.cursor.role_leaf_child::<End, _>(&ctx.trav);

        // compare next child
        if path_leaf == query_leaf {
            TDNext::Matched(self.on_match(ctx))
        } else if path_leaf.width() == 1 && query_leaf.width() == 1 {
            TDNext::Mismatched(self.on_mismatch(&ctx.trav))
        } else {
            TDNext::Prefixes(StateNext {
                prev: key.to_prev(0),
                inner: match path_leaf.width.cmp(&query_leaf.width) {
                    Ordering::Equal => self
                        .clone()
                        .tap_mut(|s| s.mode = PathPairMode::GraphMajor)
                        .prefix_states(&ctx.trav)
                        .into_iter()
                        .chain(
                            self.tap_mut(|s| s.mode = PathPairMode::QueryMajor)
                                .prefix_states(&ctx.trav),
                        )
                        .collect_vec(),
                    Ordering::Greater => self
                        .tap_mut(|s| s.mode = PathPairMode::GraphMajor)
                        .prefix_states(&ctx.trav),
                    Ordering::Less => self
                        .tap_mut(|s| s.mode = PathPairMode::QueryMajor)
                        .prefix_states(&ctx.trav),
                },
            })
        }
    }
    /// generate child states for index prefixes
    fn prefix_states<'a, Trav: Traversable>(
        self,
        trav: &Trav,
    ) -> Vec<ChildState> {
        let leaf = self.major_leaf(&trav);
        trav.graph()
            .expect_vertex(leaf)
            .prefix_children::<Trav>()
            .iter()
            .sorted_unstable_by(|a, b| b.child.width.cmp(&a.child.width))
            .map(|sub| {
                let mut state = self.clone();
                state.push_major(leaf.to_child_location(sub.location));
                ChildState {
                    target: DirectedKey::down(sub.child, *state.cursor.cursor_pos()),
                    ..state
                }
            })
            .collect_vec()
    }

    fn on_match<K: TraversalKind>(
        mut self,
        ctx: &mut TraversalContext<K>,
    ) -> MatchedNext {
        //let key = self.target_key();
        ctx.add_root_candidate();

        let path = &mut self.base.path;
        if path.advance(&ctx.trav).is_continue() {
            // gen next child
            MatchedNext::NextChild(ChildState {
                target: DirectedKey::down(
                    path.role_leaf_child::<End, _>(&ctx.trav),
                    *self.cursor_pos(),
                ),
                ..self
            })
        } else {
            MatchedNext::MatchedParent(self)
        }
    }
    fn on_mismatch<'a, Trav: Traversable>(
        self,
        trav: &Trav,
    ) -> StateNext<EndState> {
        let key = self.target_key();
        let BaseState {
            mut cursor,
            mut path,
            mut root_pos,
            prev_pos,
        } = self.base;
        cursor.retract(trav);
        path.retract(trav);
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
            StateNext {
                prev: key.to_prev(0),
                inner: EndState {
                    root_pos,
                    cursor: cursor.clone(),
                    reason: EndReason::Mismatch,
                    kind: EndKind::Complete(index),
                },
            }
        } else {
            StateNext {
                prev: key.to_prev(0),
                inner: EndState {
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
                },
            }
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
