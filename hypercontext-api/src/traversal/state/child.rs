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
    path::{
        accessors::{
            role::{
                End,
                Start,
            },
            root::GraphRoot,
        },
        mutators::{
            lower::PathLower,
            move_path::{
                key::{
                    AdvanceKey,
                    TokenPosition,
                },
                Advance,
                Retract,
            },
        },
        structs::pair::{
            PathPair,
            PathPairMode,
        },
    },
    traversal::{
        cache::{
            entry::new::NewEntry,
            key::{
                pos::CursorPosition,
                prev::ToPrev,
                target::TargetKey,
                DirectedKey,
            },
        },
        container::pruning::PruneStates,
        result::kind::{
            Primer,
            RoleChildPath,
        },
        state::{
            end::{
                EndKind,
                EndReason,
                EndState,
                RangeEnd,
            },
            parent::ParentState,
            NextStates,
            StateNext,
        },
        traversable::{
            TravDir,
            Traversable,
        },
        TraversalContext,
        TraversalKind,
    },
};
use itertools::Itertools;
use std::cmp::Ordering;
use tap::Tap;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChildState {
    pub prev_pos: TokenPosition,
    pub root_pos: TokenPosition,
    pub target: DirectedKey,
    pub paths: PathPair,
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
        self.paths
            .path
            .root_parent()
            .cmp(&other.paths.path.root_parent())
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

impl ChildState {
    pub fn next_states<K: TraversalKind>(
        self,
        ctx: &mut TraversalContext<'_, K>,
        new: Vec<NewEntry>,
    ) -> NextStates {
        let key = self.target_key();
        let path_leaf = self.paths.path.role_leaf_child::<End, _>(ctx.trav);
        let query_leaf = self.paths.cursor.role_leaf_child::<End, _>(ctx.trav);

        // compare next child
        match path_leaf.width.cmp(&query_leaf.width) {
            Ordering::Equal => {
                if path_leaf == query_leaf {
                    self.on_match(ctx, new)
                } else if path_leaf.width() == 1 {
                    self.on_mismatch(ctx.trav, new)
                } else {
                    // expand states to find matching prefix
                    NextStates::Prefixes(StateNext {
                        prev: key.to_prev(0),
                        new,
                        inner: self
                            .clone()
                            .tap_mut(|s| s.paths.mode = PathPairMode::GraphMajor)
                            .prefix_states(ctx.trav, path_leaf)
                            .into_iter()
                            .chain(
                                self.tap_mut(|s| s.paths.mode = PathPairMode::QueryMajor)
                                    .prefix_states(ctx.trav, query_leaf),
                            )
                            .collect_vec(),
                    })
                }
            }
            Ordering::Greater =>
            // continue in prefix of child
            {
                NextStates::Prefixes(StateNext {
                    prev: key.to_prev(0),
                    new,
                    inner: self
                        .tap_mut(|s| s.paths.mode = PathPairMode::GraphMajor)
                        .prefix_states(ctx.trav, path_leaf),
                })
            }
            Ordering::Less => NextStates::Prefixes(StateNext {
                prev: key.to_prev(0),
                new,
                inner: self
                    .tap_mut(|s| s.paths.mode = PathPairMode::QueryMajor)
                    .prefix_states(ctx.trav, query_leaf),
            }),
        }
    }
    fn on_match<K: TraversalKind>(
        mut self,
        ctx: &mut TraversalContext<'_, K>,
        new: Vec<NewEntry>,
    ) -> NextStates {
        let key = self.target_key();
        ctx.states.clear();
        for entry in new {
            ctx.states.cache.add_state(ctx.trav, entry, true);
        }
        //query.cache.add_path(
        //    self.trav(),
        //    path.end_path(),
        //    root_pos,
        //    false,
        //);
        let path = &mut self.paths.path;
        let qres = self.paths.cursor.advance(ctx.trav);
        if qres.is_continue() {
            if path.advance(ctx.trav).is_continue() {
                // gen next child
                NextStates::Child(StateNext {
                    prev: key.to_prev(0),
                    new: vec![],
                    inner: ChildState {
                        prev_pos: self.prev_pos,
                        root_pos: self.root_pos,
                        target: DirectedKey::down(
                            path.role_leaf_child::<End, _>(ctx.trav),
                            *self.cursor_pos(),
                        ),
                        paths: self.paths,
                    },
                })
            } else {
                ParentState {
                    prev_pos: self.prev_pos,
                    root_pos: self.root_pos,
                    path: Primer::from(self.paths.path),
                    cursor: self.paths.cursor,
                }
                .next_parents::<K>(ctx.trav, vec![])
            }
        } else {
            self.on_query_end(ctx.trav, vec![])
        }
    }
    fn on_mismatch<'a, Trav: Traversable>(
        mut self,
        trav: &Trav,
        new: Vec<NewEntry>,
    ) -> NextStates {
        let key = self.target_key();
        let PathPair {
            mut cursor,
            mut path,
            ..
        } = self.paths;
        cursor.retract(trav);
        path.retract(trav);
        if let Some(index) = loop {
            if path.role_root_child_pos::<Start>() == path.role_root_child_pos::<End>() {
                if (&mut self.root_pos, &mut path).path_lower(trav).is_break() {
                    let graph = trav.graph();
                    let pattern = graph.expect_pattern_at(path.root.location);
                    let entry = path.start.sub_path.root_entry;
                    self.root_pos = self.prev_pos;
                    break Some(pattern[entry]);
                }
            } else {
                break None;
            }
        } {
            NextStates::End(StateNext {
                prev: key.to_prev(0),
                new,
                inner: EndState {
                    root_pos: self.root_pos,
                    cursor: cursor.clone(),
                    reason: EndReason::Mismatch,
                    kind: EndKind::Complete(index),
                },
            })
        } else {
            NextStates::End(StateNext {
                prev: key.to_prev(0),
                new,
                inner: EndState {
                    root_pos: self.root_pos,
                    reason: EndReason::Mismatch,
                    kind: RangeEnd {
                        target: DirectedKey::down(
                            path.role_leaf_child::<End, _>(trav),
                            cursor.relative_pos,
                        ),
                        path,
                    }
                    .simplify(trav),
                    cursor: cursor.clone(),
                },
            })
        }
    }
    fn on_query_end<'a, Trav: Traversable>(
        self,
        trav: &Trav,
        new: Vec<NewEntry>,
    ) -> NextStates {
        let key = self.target_key();
        let PathPair {
            mut cursor, path, ..
        } = self.paths;
        let target_index = path.role_leaf_child::<End, _>(trav);
        let pos = cursor.relative_pos;
        cursor.advance_key(target_index.width());
        NextStates::End(StateNext {
            prev: key.to_prev(0),
            new,
            inner: EndState {
                root_pos: self.root_pos,
                cursor,
                reason: EndReason::QueryEnd,
                kind: RangeEnd {
                    path,
                    target: DirectedKey::down(target_index, pos),
                }
                .simplify(trav),
            },
        })
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
                let mut paths = self.paths.clone();
                paths.push_major(ChildLocation::new(index, pid, sub_index));
                ChildState {
                    prev_pos: self.prev_pos,
                    root_pos: self.root_pos,
                    target: DirectedKey::down(
                        paths.path.role_leaf_child::<End, _>(trav),
                        *paths.cursor.cursor_pos(),
                    ),
                    paths,
                }
            })
            .collect_vec()
    }
}
