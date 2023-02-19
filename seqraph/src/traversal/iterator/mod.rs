pub mod bands;

pub use bands::*;

use crate::*;

use super::*;

#[derive(Clone, Debug)]
pub struct NextMatched<T> {
    pub prev: CacheKey,
    pub inner: T,
    pub matched: bool,
    //pub query: QueryState,
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
                        matched: state.matched,
                        kind: InnerKind::Parent(s.clone())
                    })
                    .collect_vec(),
            Self::Prefixes(state) =>
                state.inner.iter()
                    .map(|s|
                        TraversalState {
                            prev: state.prev,
                            matched: state.matched,
                            kind: InnerKind::Child(s.clone()),
                        }
                    )
                    .collect_vec(),
            Self::End(state) =>
                vec![TraversalState {
                    prev: state.prev,
                    matched: state.matched,
                    kind: InnerKind::End(state.inner),
                }],
            Self::Child(state) =>
                vec![TraversalState {
                    prev: state.prev,
                    matched: state.matched,
                    kind: InnerKind::Child(state.inner),
                }],
            Self::Empty => vec![],
        }
    }
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
                matched: false,
                inner: S::gen_parent_states(
                    self.trav(),
                    &query.state,
                    state.index,
                    |trav, p|
                        MatchEnd::Complete(state.index)
                            .into_primer(trav, p)
                ),
            })
        } else {
            NextStates::End(NextMatched {
                prev: key,
                matched: false,
                inner: EndState {
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
        let (depth, tstate) = self.next()?;
        let cached = cache.add_state(self.trav(), &tstate);
        let prev = tstate.prev_key();
        let next_states = match tstate.kind {
            InnerKind::Parent(state) =>
                match cached {
                    Ok(key) =>
                        self.on_parent(
                            cache,
                            key,
                            tstate.matched,
                            state,
                        ),
                    Err(key) => {
                        cache.get_entry_mut(&key)
                            .unwrap()
                            .add_waiting(depth, WaitingState {
                                prev: tstate.prev,
                                matched: tstate.matched,
                                //query: state.query,
                                state,
                            });
                        NextStates::Empty
                    }
                },
            InnerKind::Child(state) => {
                let key = match cached {
                    Ok(key) => key,
                    Err(key) => key,
                };
                self.on_child(
                    cache,
                    prev,
                    key,
                    tstate.matched,
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
        cache: &mut TraversalCache,
        key: CacheKey,
        matched: bool,
        state: ParentState,
        //query: CachedQuery<'_>,
    ) -> NextStates {
        //let query = state.query;
        match state.path.into_advanced(self.trav()) {
            // first child state in this parent
            Ok(path) => NextStates::Child(
                NextMatched {
                    prev: key,
                    matched,
                    //query: query.state,
                    inner: ChildState {
                        //root: key,
                        paths: PathPair::new(
                            path,
                            state.query,
                            PathPairMode::GraphMajor,
                        )
                    }
                },
            ),
            // no child state, bottom up path at end of parent
            Err(path) => self.next_parents(
                path,
                matched,
                key,
                state.query.to_cached(cache),
            )
        }
    }
    fn next_parents(
        &mut self,
        primer: Primer,
        matched: bool,
        key: CacheKey,
        query: CachedQuery<'_>,
    ) -> NextStates {
        // get next parents
        let postfix = primer.into();
        let parents = S::next_parents(
            self.trav(),
            &postfix,
            &query.state,
        );
        if parents.is_empty() {
            NextStates::End(NextMatched {
                prev: key,
                matched,
                inner: EndState {
                    // todo: proxy state to delay simplification
                    kind: EndKind::from(postfix),
                    query: query.state,
                },
            })
        } else {
            NextStates::Parents(NextMatched {
                prev: key,
                matched,
                inner: parents,
                //query: query.state,
            })
        }
    }
    /// match query position with graph position
    fn on_child(
        &mut self,
        cache: &mut TraversalCache,
        prev: CacheKey,
        key: CacheKey,
        matched: bool,
        state: ChildState,
    ) -> NextStates {
        let ChildState {
            //root,
            paths,
        } = state;
        let PathPair {
            query,
            path,
            mode,
        } = paths;
        let query = query.to_cached(cache);
        let path_leaf = path.role_leaf_child::<End, _>(self.trav());
        let query_leaf = query.role_leaf_child::<End, _>(self.trav());

        // compare next child
        match path_leaf.width.cmp(&query_leaf.width) {
            Ordering::Equal =>
                if path_leaf == query_leaf {
                    self.on_match(
                        mode,
                        path,
                        query,
                        prev,
                        key,
                        //root,
                    )
                } else if path_leaf.width() == 1 {
                    self.on_mismatch(
                        matched,
                        path,
                        query.state,
                        key,
                    )
                } else {
                    // expand states to find matching prefix
                    NextStates::Prefixes(NextMatched {
                        prev: key,
                        matched,
                        //query: query.state,
                        inner: self.prefix_states(
                            //root, 
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
                                //root,
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
                    matched,
                    //query: query.state,
                    inner: self.prefix_states(
                        //root,
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
                    matched,
                    //query: query.state,
                    inner: self.prefix_states(
                        //root, 
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
        mode: PathPairMode,
        mut path: SearchPath,
        mut query: CachedQuery<'_>,
        prev: CacheKey,
        key: CacheKey,
        //root: CacheKey,
    ) -> NextStates {
        if query.advance(self.trav()).is_continue() {
            if path.advance(self.trav()).is_continue() {
                NextStates::Child(NextMatched {
                    prev,
                    matched: true,
                    inner: ChildState {
                        //root,
                        paths: PathPair::new(path, query.state, mode)
                    }
                })
            } else {
                self.next_parents(
                    Primer::from(path),
                    true,
                    key,
                    query
                )
            }
        } else {
            //path.child_path_mut::<End>().simplify::<_, D, _>(self.trav());
            self.on_range_end(
                true,
                path,
                query.state,
                prev,
                RangeKind::QueryEnd
            )
        }
    }
    fn on_range_end(
        &mut self,
        matched: bool,
        path: SearchPath,
        query: QueryState,
        key: CacheKey,
        range_kind: RangeKind,
    ) -> NextStates {
        NextStates::End(NextMatched {
            prev: key,
            matched,
            inner: EndState {
                query,
                kind: EndKind::Range(RangeEnd {
                    kind: range_kind,
                    path,
                })
            }
        })
    }
    fn on_mismatch(
        &mut self,
        matched: bool,
        mut path: SearchPath,
        query: QueryState,
        key: CacheKey,
    ) -> NextStates {
        path.retract(self.trav());
        self.on_range_end(
            matched,
            path,
            query,
            key,
            RangeKind::Mismatch
        )
    }
    /// generate child states for index prefixes
    fn prefix_states(
        &self,
        //root: CacheKey,
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
                    //root,
                    paths,
                }
            })
            .collect_vec()
    }
}