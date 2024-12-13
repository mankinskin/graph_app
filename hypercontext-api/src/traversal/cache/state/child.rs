use crate::{
    direction::r#match::MatchDirection, 
    graph::vertex::pattern::Pattern,
    traversal::{
        cache::{
            entry::new::NewEntry,
            key::{
                pos::QueryPosition, prev::ToPrev, target::TargetKey, DirectedKey
            },
            state::{
                end::{
                    EndKind,
                    EndReason,
                    EndState,
                    RangeEnd,
                }, parent::ParentState, NextStates, StateNext
            },
        },
        context::TraversalContext,
        iterator::{
            traverser::pruning::PruneStates, TraversalIterator
        },
        result::kind::{
            Primer,
            RoleChildPath,
        },
        traversable::{
            DirectionOf,
            Traversable,
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
                }, Advance, Retract
            },
        },
        structs::pair::{
            PathPair,
            PathPairMode,
        },
    },
};
use itertools::Itertools;
use std::cmp::Ordering;
use tap::Tap;
use crate::graph::vertex::{
    child::Child,
    location::child::ChildLocation,
    wide::Wide,
};
use crate::graph::getters::vertex::VertexSet;

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
    pub fn next_states<'a, 'b: 'a, I: TraversalIterator<'b>>(
        mut self,
        ctx: &mut TraversalContext<'a, 'b, I>,
        new: Vec<NewEntry>,
    ) -> NextStates {
        let key = self.target_key();
        let query = self.paths.query.to_ctx(ctx);
        let path_leaf = self.paths.path.role_leaf_child::<End, _>(ctx.trav());
        let query_leaf = query.role_leaf_child::<End, _>(ctx.trav());

        // compare next child
        match path_leaf.width.cmp(&query_leaf.width) {
            Ordering::Equal => {
                if path_leaf == query_leaf {
                    self.on_match(ctx, new)
                } else if path_leaf.width() == 1 {
                    self.on_mismatch(ctx, new)
                } else {
                    // expand states to find matching prefix
                    NextStates::Prefixes(StateNext {
                        prev: key.to_prev(0),
                        new,
                        inner: self
                            .clone()
                            .tap_mut(|s| s.paths.mode = PathPairMode::GraphMajor)
                            .prefix_states(ctx, path_leaf)
                            .into_iter()
                            .chain(
                                self.tap_mut(|s| s.paths.mode = PathPairMode::QueryMajor)
                                    .prefix_states(ctx, query_leaf),
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
                        .prefix_states(ctx, path_leaf),
                })
            }
            Ordering::Less => NextStates::Prefixes(StateNext {
                prev: key.to_prev(0),
                new,
                inner: self
                    .tap_mut(|s| s.paths.mode = PathPairMode::QueryMajor)
                    .prefix_states(ctx, query_leaf),
            }),
        }
    }
    fn on_match<'a, 'b: 'a, I: TraversalIterator<'b>>(
        mut self,
        ctx: &mut TraversalContext<'a, 'b, I>,
        new: Vec<NewEntry>,
    ) -> NextStates {
        let key = self.target_key();
        ctx.clear();
        for entry in new {
            ctx.cache.add_state(ctx.trav(), entry, true);
        }
        //query.cache.add_path(
        //    self.trav(),
        //    path.end_path(),
        //    root_pos,
        //    false,
        //);
        let path = &mut self.paths.path;
        let mut query = self.paths.query.to_ctx(ctx);
        let qres = query.advance(ctx.trav());
        if qres.is_continue() {
            if path.advance(ctx.trav()).is_continue() {
                // gen next child
                NextStates::Child(StateNext {
                    prev: key.to_prev(0),
                    new: vec![],
                    inner: ChildState {
                        prev_pos: self.prev_pos,
                        root_pos: self.root_pos,
                        target: DirectedKey::down(
                            path.role_leaf_child::<End, _>(ctx.trav()),
                            *self.paths.query.query_pos(),
                        ),
                        paths: self.paths,
                    },
                })
            } else {
                ParentState {
                    prev_pos: self.prev_pos,
                    root_pos: self.root_pos,
                    path: Primer::from(self.paths.path),
                    query: self.paths.query,
                }
                .next_parents(ctx, vec![])
            }
        } else {
            self.on_query_end(ctx, vec![])
        }
    }
    fn on_mismatch<'a, 'b: 'a, I: TraversalIterator<'b>>(
        mut self,
        ctx: &mut TraversalContext<'a, 'b, I>,
        new: Vec<NewEntry>,
    ) -> NextStates {
        let key = self.target_key();
        let PathPair {
            mut query,
            mut path,
            ..
        } = self.paths;
        let mut query = query.to_ctx(ctx);
        query.retract(ctx.trav());
        path.retract(ctx.trav());
        if let Some(index) = loop {
            if path.role_root_child_pos::<Start>() == path.role_root_child_pos::<End>() {
                if (&mut self.root_pos, &mut path)
                    .path_lower(ctx.trav())
                    .is_break()
                {
                    let graph = ctx.trav().graph();
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
                    query: query.state.clone(),
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
                            path.role_leaf_child::<End, _>(ctx.trav()),
                            query.state.pos,
                        ),
                        path,
                    }
                    .simplify(ctx.trav()),
                    query: query.state.clone(),
                },
            })
        }
    }
    fn on_query_end<'a, 'b: 'a, I: TraversalIterator<'b>>(
        self,
        ctx: &mut TraversalContext<'a, 'b, I>,
        new: Vec<NewEntry>,
    ) -> NextStates {
        let key = self.target_key();
        let PathPair {
            mut query, path, ..
        } = self.paths;
        let target_index = path.role_leaf_child::<End, _>(ctx.trav());
        let pos = query.pos;
        query.advance_key(target_index.width());
        NextStates::End(StateNext {
            prev: key.to_prev(0),
            new,
            inner: EndState {
                root_pos: self.root_pos,
                query,
                reason: EndReason::QueryEnd,
                kind: RangeEnd {
                    path,
                    target: DirectedKey::down(target_index, pos),
                }
                .simplify(ctx.trav()),
            },
        })
    }
    /// generate child states for index prefixes
    fn prefix_states<'a, I: TraversalIterator<'a>>(
        self,
        ctx: &mut I,
        index: Child,
    ) -> Vec<ChildState> {
        ctx.trav()
            .graph()
            .expect_vertex(index)
            .get_child_patterns()
            .iter()
            .sorted_unstable_by(|(_, a), (_, b)| {
                b.first().unwrap().width.cmp(&a.first().unwrap().width)
            })
            .map(|(&pid, child_pattern): (_, &Pattern)| {
                let sub_index = DirectionOf::<I::Trav>::head_index(child_pattern);
                let mut paths = self.paths.clone();
                paths.push_major(ChildLocation::new(index, pid, sub_index));
                ChildState {
                    prev_pos: self.prev_pos,
                    root_pos: self.root_pos,
                    target: DirectedKey::down(
                        paths.path.role_leaf_child::<End, _>(ctx.trav()),
                        *paths.query.query_pos(),
                    ),
                    paths,
                }
            })
            .collect_vec()
    }
}
