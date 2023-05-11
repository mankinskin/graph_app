pub mod bands;

pub use bands::*;

use crate::*;

use super::*;

#[derive(Clone, Debug)]
pub struct NextMatched<T> {
    pub prev: DirectedKey,
    pub new: Vec<NewEntry>,
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
                        new: state.new.clone(),
                        kind: InnerKind::Parent(s.clone())
                    })
                    .collect_vec(),
            Self::Prefixes(state) =>
                state.inner.iter()
                    .map(|s|
                        TraversalState {
                            prev: state.prev,
                            new: state.new.clone(),
                            kind: InnerKind::Child(s.clone()),
                        }
                    )
                    .collect_vec(),
            Self::End(_) =>
                vec![
                    //TraversalState {
                    //    prev: state.prev,
                    //    new: state.new,
                    //    kind: InnerKind::End(state.inner),
                    //}
                ],
            Self::Child(state) =>
                vec![TraversalState {
                    prev: state.prev,
                    new: state.new,
                    kind: InnerKind::Child(state.inner),
                }],
            Self::Empty => vec![],
        }
    }
}
#[derive(Clone, Debug)]
pub struct EdgeKeys {
    pub key: DirectedKey,
    pub prev: DirectedKey,
}
pub trait TraversalIterator<
    'a, 
    Trav: Traversable + 'a + TraversalFolder<S>,
    S: DirectedTraversalPolicy<Trav=Trav>,
>: Iterator<Item = (usize, TraversalState)> + Sized + ExtendStates + PruneStates
{

    fn new(trav: &'a Trav) -> Self;
    fn trav(&self) -> &'a Trav;

    /// states generated from a query start state
    /// (query end or start parent states)
    fn query_start(
        &self,
        key: DirectedKey,
        mut query: CachedQuery<'_>,
        state: StartState,
    ) -> NextStates {
        if query.advance(self.trav()).is_continue() {
            // undo extra key advance
            query.retract_key(state.index.width());
            NextStates::Parents(NextMatched {
                prev: key,
                new: vec![],
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
                new: vec![],
                inner: EndState {
                    reason: EndReason::QueryEnd,
                    root_pos: state.index.width().into(),
                    kind: EndKind::Complete(state.index),
                    query: query.state,
                }
            })
        }
    }
    fn next_states(
        &mut self,
        cache: &mut TraversalCache,
    ) -> Option<(usize, NextStates)> {
        let (depth, mut tstate) = self.next()?;
        let key = tstate.target_key();
        let exists = cache.exists(&key);
        //let prev = tstate.prev_key();
        //if !exists {
        //    cache.add_state((&tstate).into());
        //}
        if !exists && matches!(tstate.kind, InnerKind::Parent(_)) {
            tstate.new.push((&tstate).into());
        }
        let next_states = match tstate.kind {
            InnerKind::Parent(state) => {
                //debug!("Parent({}, {})", key.index.index(), key.index.width());
                if !exists {
                    self.on_parent(
                        key,
                        state,
                        tstate.new,
                    )
                } else {
                    //cache.get_mut(&key)
                    //    .unwrap()
                    //    .add_waiting(depth, WaitingState {
                    //        prev,
                    //        state,
                    //    });
                    for entry in tstate.new {
                        cache.add_state(entry, true);
                    }
                    NextStates::Empty
                }
            },
            InnerKind::Child(state) => {
                //let root = state.paths.path.root_parent();
                //debug!("Child({}, {}), root = ({}, {})",
                //    key.index.index(),
                //    key.index.width(),
                //    root.index(),
                //    root.width(),
                //);
                if !exists {
                    self.on_child(
                        cache,
                        key,
                        state,
                        tstate.new,
                    )
                } else {
                    cache.add_path(
                        self.trav(),
                        state.paths.path.role_root_child_location::<Start>().sub_index,
                        &state.paths.path,
                        state.root_pos,
                        false,
                    );
                    NextStates::Empty
                }
            }
        };
        Some((depth + 1, next_states))
    }
    fn on_parent(
        &mut self,
        key: DirectedKey,
        state: ParentState,
        new: Vec<NewEntry>,
    ) -> NextStates {
        match state.into_advanced(self.trav()) {
            // first child state in this parent
            Ok(advanced) => NextStates::Child(
                NextMatched {
                    prev: key.flipped(),
                    new,
                    inner: advanced
                },
            ),
            // no child state, bottom up path at end of parent
            Err(state) => self.next_parents(
                state,
                key,
                new,
            )
        }
    }
    /// match query position with graph position
    fn on_child(
        &mut self,
        cache: &mut TraversalCache,
        key: DirectedKey,
        state: ChildState,
        new: Vec<NewEntry>,
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
                        key,
                        new,
                    )
                } else if path_leaf.width() == 1 {
                    self.on_mismatch(
                        state.prev_pos,
                        state.root_pos,
                        path,
                        query,
                        key,
                        new,
                    )
                } else {
                    // expand states to find matching prefix
                    NextStates::Prefixes(NextMatched {
                        prev: key,
                        new,
                        inner: self.prefix_states(
                            state.prev_pos,
                            state.root_pos,
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
                    prev: key,
                    new,
                    inner: self.prefix_states(
                        state.prev_pos,
                        state.root_pos,
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
                    prev: key,
                    new,
                    inner: self.prefix_states(
                        state.prev_pos,
                        state.root_pos,
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
        key: DirectedKey,
        new: Vec<NewEntry>,
    ) -> NextStates {
        self.clear();
        for entry in new {
            query.cache.add_state(entry, true);
        }
        //query.cache.add_path(
        //    self.trav(),
        //    path.end_path(),
        //    root_pos,
        //    false,
        //);
        if query.advance(self.trav()).is_continue() {
            if path.advance(self.trav()).is_continue() {
                // gen next child
                NextStates::Child(NextMatched {
                    prev: key,
                    new: vec![],
                    inner: ChildState {
                        prev_pos,
                        root_pos,
                        target: DirectedKey::down(
                            path.role_leaf_child::<End, _>(self.trav()),
                            *query.state.query_pos()
                        ),
                        paths: PathPair::new(
                            path,
                            query.state,
                            mode,
                        ),
                    }
                })
            } else {
                self.next_parents(
                    ParentState {
                        prev_pos,
                        root_pos,
                        path: Primer::from(path),
                        query: query.state,
                    },
                    key,
                    vec![],
                )
            }
        } else {
            //path.child_path_mut::<End>().simplify(self.trav());
            self.on_range_end(
                root_pos,
                path,
                query,
                key,
                EndReason::QueryEnd,
                vec![],
            )
        }
    }
    fn next_parents(
        &mut self,
        parent: ParentState,
        key: DirectedKey,
        new: Vec<NewEntry>
    ) -> NextStates {
        // get next parents
        let parents = S::next_parents(
            self.trav(),
            &parent,
        );
        if parents.is_empty() {
            NextStates::End(NextMatched {
                prev: key,
                new,
                inner: EndState {
                    reason: EndReason::Mismatch,
                    root_pos: parent.root_pos,
                    kind: parent.path.simplify(self.trav()),
                    query: parent.query,
                },
            })
        } else {
            NextStates::Parents(NextMatched {
                prev: key,
                new,
                inner: parents,
            })
        }
    }
    fn on_range_end(
        &mut self,
        root_pos: TokenLocation,
        path: SearchPath,
        mut query: CachedQuery<'_>,
        key: DirectedKey,
        reason: EndReason,
        new: Vec<NewEntry>,
    ) -> NextStates {
        let target_index = path.role_leaf_child::<End, _>(self.trav());
        let pos = query.state.pos;
        query.advance_key(target_index.width());
        NextStates::End(NextMatched {
            prev: key,
            new,
            inner: EndState {
                root_pos,
                query: query.state,
                reason,
                kind: RangeEnd {
                    path,
                    target: DirectedKey::down(
                        target_index,
                        pos,
                    ),
                }.simplify(self.trav()),
            }
        })
    }
    fn on_mismatch(
        &mut self,
        prev_pos: TokenLocation,
        mut root_pos: TokenLocation,
        mut path: SearchPath,
        mut query: CachedQuery<'_>,
        key: DirectedKey,
        new: Vec<NewEntry>,
    ) -> NextStates {
        query.retract(self.trav());
        path.retract(self.trav());
        if let Some(index) = loop {
            if path.role_root_child_pos::<Start>() == path.role_root_child_pos::<End>() {
                if (&mut root_pos, &mut path).path_lower(self.trav()).is_break() {
                    let graph = self.trav().graph();
                    let pattern = graph.expect_pattern_at(&path.root.location);
                    let entry = path.start.sub_path.root_entry;
                    root_pos = prev_pos;
                    break Some(pattern[entry]);
                }
            } else {
                break None;
            }
        } {
            NextStates::End(NextMatched {
                prev: key,
                new,
                inner: EndState {
                    root_pos,
                    query: query.state,
                    reason: EndReason::Mismatch,
                    kind: EndKind::Complete(index),
                }
            })
        } else {
            NextStates::End(NextMatched {
                prev: key,
                new,
                inner: EndState {
                    root_pos,
                    reason: EndReason::Mismatch,
                    kind: RangeEnd {
                        target: DirectedKey::down(
                            path.role_leaf_child::<End, _>(self.trav()),
                            query.state.pos,
                        ),
                        path,
                    }.simplify(self.trav()),
                    query: query.state,
                },
            })
        }
    }
    /// generate child states for index prefixes
    fn prefix_states(
        &self,
        prev_pos: TokenLocation,
        root_pos: TokenLocation,
        index: Child,
        paths: PathPair,
    ) -> Vec<ChildState> {
        self.trav().graph()
            .expect_vertex_data(index)
            .get_child_patterns().iter()
            .sorted_unstable_by(|(_, a), (_, b)|
                b.first().unwrap().width.cmp(
                    &a.first().unwrap().width
                )
            )
            .map(|(&pid, child_pattern)| {
                let sub_index = <Trav::Kind as GraphKind>::Direction::head_index(
                    child_pattern.borrow() as &[Child]
                );
                let mut paths = paths.clone();
                paths.push_major(ChildLocation::new(index, pid, sub_index));
                ChildState {
                    prev_pos,
                    root_pos,
                    target: DirectedKey::down(
                        paths.path.role_leaf_child::<End, _>(self.trav()),
                        *paths.query.query_pos(),
                    ),
                    paths,
                }
            })
            .collect_vec()
    }
}