pub mod bands;

pub use bands::*;

use crate::*;

use super::*;

#[derive(Clone, Debug)]
pub struct NextMatched<T> {
    pub prev: CacheKey,
    pub inner: T,
}
#[derive(Clone, Debug)]
pub enum NextStates {
    Parents(NextMatched<Vec<ParentState>>),
    Prefixes(NextMatched<Vec<ChildState>>),
    End(NextMatched<EndState>),
    Child(NextMatched<ChildState>),
    Empty,
}
impl NextStates {
    pub fn into_states(self) -> Vec<TraversalState> {
        match self {
            Self::Parents(state) =>
                state.inner.iter()
                    .map(|s| TraversalState {
                        prev: state.prev,
                        kind: InnerKind::Parent(s.clone())
                    })
                    .collect_vec(),
            Self::Prefixes(state) =>
                state.inner.iter()
                    .map(|s|
                        TraversalState {
                            prev: state.prev,
                            kind: InnerKind::Child(s.clone()),
                        }
                    )
                    .collect_vec(),
            Self::End(state) =>
                vec![TraversalState {
                    prev: state.prev,
                    kind: InnerKind::End(state.inner),
                }],
            Self::Child(state) =>
                vec![TraversalState {
                    prev: state.prev,
                    kind: InnerKind::Child(state.inner),
                }],
            Self::Empty => vec![],
        }
    }
}
#[derive(Clone, Debug)]
pub struct EdgeKeys {
    pub key: CacheKey,
    pub prev: CacheKey,
}
pub trait TraversalIterator<
    'a, 
    Trav: Traversable + 'a + TraversalFolder<S>,
    S: DirectedTraversalPolicy<Trav=Trav>,
>: Iterator<Item = (usize, TraversalState)> + Sized + ExtendStates
{

    fn new(trav: &'a Trav) -> Self;
    fn trav(&self) -> &'a Trav;

    /// states generated from a query start state
    /// (query end or start parent states)
    fn query_start(
        &self,
        key: CacheKey,
        mut query: CachedQuery<'_>,
        state: StartState,
    ) -> NextStates {
        if query.advance(self.trav()).is_continue() {
            NextStates::Parents(NextMatched {
                prev: key,
                inner: S::gen_parent_states(
                    self.trav(),
                    state.index,
                    |trav, p|
                        (state.index, query.state.clone())
                            .into_primer(trav, p)
                ),
            })
        } else {
            NextStates::End(NextMatched {
                prev: key,
                inner: EndState {
                    reason: EndReason::QueryEnd,
                    root_pos: state.index.width().into(),
                    kind: EndKind::Complete(state.index),
                    query: query.state,
                    matched: false,
                }
            })
        }
    }
    fn next_states(
        &mut self,
        cache: &mut TraversalCache,
    ) -> Option<(usize, NextStates)> {
        let (depth, tstate) = self.next()?;
        let (key, cached) = cache.add_state(self.trav(), &tstate);
        let prev = tstate.prev_key();

        let keys = EdgeKeys { prev, key };
        let next_states = match tstate.kind {
            InnerKind::Parent(state) =>
                if cached {
                    self.on_parent(
                        key,
                        //tstate.matched,
                        state,
                    )
                } else {
                    cache.get_entry_mut(&key)
                        .unwrap()
                        .add_waiting(depth, WaitingState {
                            prev: tstate.prev,
                            //query: state.query,
                            state,
                        });
                    NextStates::Empty
                },
            InnerKind::Child(state) => {
                self.on_child(
                    cache,
                    keys,
                    state,
                )

                //        cache.get_entry_mut(&key)
                //            .unwrap()
                //            .add_back_edge();
                //        NextStates::Empty
            }
            _ => NextStates::Empty,
        };
        Some((depth + 1, next_states))
    }
    fn on_parent(
        &mut self,
        key: CacheKey,
        state: ParentState,
    ) -> NextStates {
        match state.into_advanced(self.trav()) {
            // first child state in this parent
            Ok(advanced) => NextStates::Child(
                NextMatched {
                    prev: key,
                    inner: advanced
                },
            ),
            // no child state, bottom up path at end of parent
            Err(state) => self.next_parents(
                state,
                key,
            )
        }
    }
    fn next_parents(
        &mut self,
        parent: ParentState,
        key: CacheKey,
    ) -> NextStates {
        // get next parents
        let parents = S::next_parents(
            self.trav(),
            &parent,
        );
        if parents.is_empty() {
            NextStates::End(NextMatched {
                prev: key,
                inner: EndState {
                    reason: EndReason::Mismatch,
                    root_pos: parent.root_pos,
                    kind: EndKind::from(
                        Postfix::from(parent.path)
                            .into_simplified(self.trav())
                    ),
                    matched: parent.matched,
                    query: parent.query,
                },
            })
        } else {
            NextStates::Parents(NextMatched {
                prev: key,
                inner: parents,
            })
        }
    }
    /// match query position with graph position
    fn on_child(
        &mut self,
        cache: &mut TraversalCache,
        keys: EdgeKeys,
        state: ChildState,
    ) -> NextStates {
        let PathPair {
            query,
            path,
            mode,
        } = state.paths;
        let query = query.to_cached(cache);
        let path_leaf = path.role_leaf_child::<End, _>(self.trav());
        let query_leaf = query.role_leaf_child::<End, _>(self.trav());

        // compare next child
        match path_leaf.width.cmp(&query_leaf.width) {
            Ordering::Equal =>
                if path_leaf == query_leaf {
                    self.on_match(
                        state.prev_pos,
                        state.root_pos,
                        mode,
                        path,
                        query,
                        keys,
                    )
                } else if path_leaf.width() == 1 {
                    self.on_mismatch(
                        state.prev_pos,
                        state.root_pos,
                        state.matched,
                        path,
                        query,
                        keys,
                    )
                } else {
                    // expand states to find matching prefix
                    NextStates::Prefixes(NextMatched {
                        prev: keys.key,
                        inner: self.prefix_states(
                            state.prev_pos,
                            state.root_pos,
                            state.matched,
                            path_leaf,
                            PathPair::new(
                                path.clone(),
                                query.state.clone(),
                                PathPairMode::GraphMajor,
                            ),
                        )
                        .into_iter()
                        .chain(
                            self.prefix_states(
                                state.prev_pos,
                                state.root_pos,
                                state.matched,
                                query_leaf,
                                PathPair::new(
                                    path,
                                    query.state,
                                    PathPairMode::QueryMajor,
                                ),
                            )
                        )
                        .collect_vec()
                    })
                }
            Ordering::Greater =>
                // continue in prefix of child
                NextStates::Prefixes(NextMatched {
                    prev: keys.key,
                    inner: self.prefix_states(
                        state.prev_pos,
                        state.root_pos,
                        state.matched,
                        path_leaf,
                        PathPair::new(
                            path,
                            query.state,
                            PathPairMode::GraphMajor,
                        ),
                    )
                }),
            Ordering::Less =>
                NextStates::Prefixes(NextMatched {
                    prev: keys.key,
                    inner: self.prefix_states(
                        state.prev_pos,
                        state.root_pos,
                        state.matched,
                        query_leaf,
                        PathPair::new(
                            path,
                            query.state,
                            PathPairMode::QueryMajor,
                        ),
                    )
                }),
        }
    }
    fn on_match(
        &mut self,
        prev_pos: TokenLocation,
        root_pos: TokenLocation,
        mode: PathPairMode,
        mut path: SearchPath,
        mut query: CachedQuery<'_>,
        keys: EdgeKeys,
    ) -> NextStates {
        if query.advance(self.trav()).is_continue() {
            if path.advance(self.trav()).is_continue() {
                // gen next child
                NextStates::Child(NextMatched {
                    prev: keys.key,
                    inner: ChildState {
                        prev_pos,
                        root_pos,
                        matched: true,
                        paths: PathPair::new(path, query.state, mode)
                    }
                })
            } else {
                self.next_parents(
                    ParentState {
                        prev_pos,
                        root_pos,
                        matched: true,
                        path: Primer::from(path),
                        query: query.state,
                    },
                    keys.key,
                )
            }
        } else {
            //path.child_path_mut::<End>().simplify(self.trav());
            self.on_range_end(
                prev_pos,
                root_pos,
                true,
                path,
                query.state,
                keys,
                EndReason::QueryEnd
            )
        }
    }
    fn on_range_end(
        &mut self,
        prev_pos: TokenLocation,
        root_pos: TokenLocation,
        matched: bool,
        path: SearchPath,
        query: QueryState,
        keys: EdgeKeys,
        reason: EndReason,
    ) -> NextStates {
        NextStates::End(NextMatched {
            prev: keys.key,
            inner: (EndState {
                root_pos,
                query,
                reason,
                matched,
                kind: EndKind::Range(RangeEnd {
                    path,
                })
            }, prev_pos).into_simplified(self.trav()).0
        })
    }
    fn on_mismatch(
        &mut self,
        prev_pos: TokenLocation,
        root_pos: TokenLocation,
        matched: bool,
        mut path: SearchPath,
        mut query: CachedQuery<'_>,
        keys: EdgeKeys,
    ) -> NextStates {
        path.retract(self.trav());
        query.retract(self.trav());
        self.on_range_end(
            prev_pos,
            root_pos,
            matched,
            path,
            query.state,
            keys,
            EndReason::Mismatch
        )
    }
    /// generate child states for index prefixes
    fn prefix_states(
        &self,
        prev_pos: TokenLocation,
        root_pos: TokenLocation,
        matched: bool,
        index: Child,
        paths: PathPair,
    ) -> Vec<ChildState> {
        self.trav().graph()
            .expect_vertex_data(index)
            .get_child_patterns().iter()
            .sorted_unstable_by_key(|(_, p)| p.first().unwrap().width)
            .map(|(&pid, child_pattern)| {
                let sub_index = <Trav::Kind as GraphKind>::Direction::head_index(child_pattern.borrow());
                let mut paths = paths.clone();
                paths.push_major(ChildLocation::new(index, pid, sub_index));
                ChildState {
                    prev_pos,
                    root_pos,
                    matched,
                    paths,
                }
            })
            .collect_vec()
    }
}