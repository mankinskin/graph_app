use crate::{
    direction::pattern::PatternDirection,
    graph::{
        getters::vertex::VertexSet,
        vertex::{
            child::Child,
            location::child::ChildLocation,
            pattern::Pattern,
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
                key::AdvanceKey,
                Advance,
                Retract,
            },
        },
        structs::rooted::{
            index_range::IndexRangePath,
            role_path::IndexStartPath,
        },
        RoleChildPath,
    },
    traversal::{
        cache::key::{
            directed::{
                up::UpKey,
                DirectedKey,
            },
            prev::{
                PrevKey,
                ToPrev,
            },
            props::{
                CursorPosition,
                LeafKey,
                RootKey,
                TargetKey,
            },
        },
        container::pruning::PruneStates,
        state::{
            bottom_up::parent::ParentState,
            top_down::end::{
                EndKind,
                EndReason,
                EndState,
                RangeEnd,
            },
            traversal::TraversalState,
            BaseState,
        },
        traversable::{
            TravDir,
            Traversable,
        },
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

use super::super::next_states::{
    NextStates,
    StateNext,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PathPairMode {
    GraphMajor,
    QueryMajor,
}

impl_cursor_pos! {
    CursorPosition for ChildState, self => self.cursor.relative_pos
}
//impl LeafKey for PathPair {
//    fn leaf_location(&self) -> ChildLocation {
//        self.path.leaf_location()
//    }
//}
#[derive(Clone, Debug, PartialEq, Eq, Deref, DerefMut)]
pub struct ChildState {
    #[deref]
    #[deref_mut]
    pub base: BaseState<IndexRangePath>,
    pub root_parent: ParentState,
    pub root_prev: PrevKey,
    pub target: DirectedKey,
    pub mode: PathPairMode,
}

impl ChildState {
    pub fn root_state(&self) -> (PrevKey, &ParentState) {
        (self.root_prev, &self.root_parent)
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
    ) -> NextStates {
        let key = self.target_key();
        let path_leaf = self.path.role_leaf_child::<End, _>(&ctx.trav);
        let query_leaf = self.cursor.role_leaf_child::<End, _>(&ctx.trav);

        // compare next child
        if path_leaf == query_leaf {
            self.on_match(ctx)
        } else if path_leaf.width() == 1 && path_leaf.width() == 1 {
            self.on_mismatch(&ctx.trav)
        } else {
            NextStates::Prefixes(StateNext {
                prev: key.to_prev(0),
                inner: match path_leaf.width.cmp(&query_leaf.width) {
                    Ordering::Equal => self
                        .clone()
                        .tap_mut(|s| s.mode = PathPairMode::GraphMajor)
                        .prefix_states(&ctx.trav, path_leaf)
                        .into_iter()
                        .chain(
                            self.tap_mut(|s| s.mode = PathPairMode::QueryMajor)
                                .prefix_states(&ctx.trav, query_leaf),
                        )
                        .collect_vec(),
                    Ordering::Greater => self
                        .tap_mut(|s| s.mode = PathPairMode::GraphMajor)
                        .prefix_states(&ctx.trav, path_leaf),
                    Ordering::Less => self
                        .tap_mut(|s| s.mode = PathPairMode::QueryMajor)
                        .prefix_states(&ctx.trav, query_leaf),
                },
            })
        }
    }
    /// generate child states for index prefixes
    fn prefix_states<'a, Trav: Traversable>(
        self,
        trav: &Trav,
        index: Child,
    ) -> Vec<ChildState> {
        trav.graph()
            .expect_vertex(index)
            .get_child_patterns()
            .iter()
            .sorted_unstable_by(|(_, a), (_, b)| {
                b.first().unwrap().width.cmp(&a.first().unwrap().width)
            })
            .map(|(&pid, child_pattern): (_, &Pattern)| {
                let sub_index = TravDir::<Trav>::head_index(child_pattern);
                let mut state = self.clone();
                state.push_major(ChildLocation::new(index, pid, sub_index));
                ChildState {
                    target: DirectedKey::down(
                        state.path.role_leaf_child::<End, _>(trav),
                        *state.cursor.cursor_pos(),
                    ),
                    ..state
                }
            })
            .collect_vec()
    }
    fn on_match<K: TraversalKind>(
        mut self,
        ctx: &mut TraversalContext<K>,
    ) -> NextStates {
        let key = self.target_key();
        ctx.states.clear();
        ctx.cache.add_state(
            &ctx.trav,
            TraversalState::from((|(p, ps): (_, &ParentState)| (p, ps.clone()))(
                self.root_state(),
            )),
            true,
        );
        //query.cache.add_path(
        //    self.trav(),
        //    path.end_path(),
        //    root_pos,
        //    false,
        //);
        let path = &mut self.base.path;
        let qres = self.base.cursor.advance(&ctx.trav);
        if qres.is_continue() {
            if path.advance(&ctx.trav).is_continue() {
                // gen next child
                NextStates::Child(StateNext {
                    prev: key.to_prev(0),
                    inner: ChildState {
                        target: DirectedKey::down(
                            path.role_leaf_child::<End, _>(&ctx.trav),
                            *self.cursor_pos(),
                        ),
                        ..self
                    },
                })
            } else {
                ParentState {
                    path: IndexStartPath::from(self.base.path),
                    ..self.base
                }
                .next_parents::<K>(&ctx.trav)
            }
        } else {
            self.on_query_end(&ctx.trav)
        }
    }
    fn on_mismatch<'a, Trav: Traversable>(
        self,
        trav: &Trav,
    ) -> NextStates {
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
            NextStates::End(StateNext {
                prev: key.to_prev(0),
                inner: EndState {
                    root_pos,
                    cursor: cursor.clone(),
                    reason: EndReason::Mismatch,
                    kind: EndKind::Complete(index),
                },
            })
        } else {
            NextStates::End(StateNext {
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
            })
        }
    }
    fn on_query_end<'a, Trav: Traversable>(
        self,
        trav: &Trav,
    ) -> NextStates {
        let key = self.target_key();
        let BaseState {
            mut cursor,
            path,
            root_pos,
            ..
        } = self.base;
        let target_index = path.role_leaf_child::<End, _>(trav);
        let pos = cursor.relative_pos;
        cursor.advance_key(target_index.width());
        NextStates::End(StateNext {
            prev: key.to_prev(0),
            inner: EndState {
                root_pos,
                cursor,
                reason: EndReason::QueryEnd,
                kind: RangeEnd {
                    path,
                    target: DirectedKey::down(target_index, pos),
                }
                .simplify_to_end(trav),
            },
        })
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
