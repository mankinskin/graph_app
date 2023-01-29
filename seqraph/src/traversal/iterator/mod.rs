pub mod bands;

pub use bands::*;

use crate::*;

use super::*;

#[derive(Clone, Debug)]
pub enum NextStates<R: ResultKind> {
    Parents(CacheKey, bool, Vec<ParentState<R>>),
    Prefixes(CacheKey, bool, Vec<ChildState<R>>),
    End(CacheKey, bool, EndState<R>),
    Child(CacheKey, bool, ChildState<R>),
    Empty,
}
impl<R: ResultKind> NextStates<R> {
    pub fn into_states(self) -> Vec<TraversalState<R>> {
        match self {
            Self::Parents(key, matched, states) =>
                states.into_iter()
                    .map(|s| TraversalState {
                        prev: key,
                        matched,
                        kind: InnerKind::Parent(s)
                    })
                    .collect_vec(),
            Self::Prefixes(key, matched, states) =>
                states.into_iter()
                    .map(|state|
                        TraversalState {
                            prev: key,
                            matched,
                            kind: InnerKind::Child(state),
                        }
                    )
                    .collect_vec(),
            Self::End(key, matched, state) =>
                vec![TraversalState {
                    prev: key,
                    matched,
                    kind: InnerKind::End(state),
                }],
            Self::Child(key, matched, state) =>
                vec![TraversalState {
                    prev: key,
                    matched,
                    kind: InnerKind::Child(state),
                }],
            Self::Empty => vec![],
        }
    }
}
pub trait TraversalIterator<
    'a, 
    T: Tokenize,
    D: MatchDirection,
    Trav: Traversable<T> + 'a + TraversalFolder<T, D, S, R>,
    S: DirectedTraversalPolicy<T, D, R, Trav=Trav>,
    R: ResultKind = BaseResult,
>: Iterator<Item = (usize, TraversalState<R>)> + Sized + ExtendStates<R>
{

    fn new(trav: &'a Trav) -> Self;
    fn trav(&self) -> &'a Trav;
    fn next_states(
        &mut self,
        cache: &mut TraversalCache<R>,
    ) -> Option<(usize, NextStates<R>)> {
        let (depth, tstate) = self.next()?;
        let next_states = match cache.add_state(self.trav(), &tstate) {
            Ok(key) => {
                match tstate.kind {
                    InnerKind::Parent(state) =>
                        self.on_parent(key, tstate.matched, state),
                    InnerKind::Child(state) =>
                        self.on_child(key, tstate.matched, state),
                    _ => NextStates::Empty,
                }
            },
            Err(key) => {
                cache.get_entry_mut(&key)
                    .unwrap()
                    .add_waiting(depth, tstate);
                NextStates::Empty
            }
        };
        Some((depth + 1, next_states))
    }
    fn on_parent(
        &mut self,
        key: CacheKey,
        matched: bool,
        state: ParentState<R>,
    ) -> NextStates<R> {
        // todo: solve the "is finished" predicate with a type (how to relate to specific trav state?)
        let query = state.query;
        //assert!(!query.is_finished(self.trav()));
        // create path to next child
        match R::Primer::into_advanced::<_, D, _>(state.path, self.trav()) {
            Ok(path) =>
                // first child state in this parent
                NextStates::Child(
                    key,
                    matched,
                    ChildState {
                        root: key,
                        paths: PathPair::GraphMajor(path, query)
                    }
                ),
            Err(primer) =>
                // no child state, bottom up path at end of parent
                self.next_parents(
                    primer,
                    matched,
                    key,
                    query,
                )
        }
    }
    fn next_parents(
        &mut self,
        primer: R::Primer,
        matched: bool,
        key: CacheKey,
        query: R::Query,
    ) -> NextStates<R> {
        // get next parents
        let postfix = primer.into();
        let parents = S::next_parents(
            self.trav(),
            &query,
            &postfix,
        );
        if parents.is_empty() {
            //println!("no more parents {:#?}", match_end);
            NextStates::End(
                key,
                matched,
                EndState {
                    root: key,
                    kind: EndKind::Postfix(PostfixEnd {
                        path: postfix,//.into_simplified::<_, D, _>(self.trav()),
                    }),
                    query,
                },
            )
        } else {
            NextStates::Parents(
                key,
                matched,
                parents
            )
        }
    }
    /// match query position with graph position
    fn on_child(
        &mut self,
        key: CacheKey,
        matched: bool,
        state: ChildState<R>,
    ) -> NextStates<R> {
        let ChildState {
            root,
            paths,
        } = state;
        let mode = paths.mode();
        let (path, query) = paths.unpack();

        let path_leaf = path.role_leaf_child::<End, _, _>(self.trav());
        let query_leaf = query.leaf_child(self.trav());

        // compare next child
        match path_leaf.width.cmp(&query_leaf.width) {
            Ordering::Equal =>
                if path_leaf == query_leaf {
                    self.on_match(
                        mode,
                        path,
                        query,
                        key,
                        root,
                    )
                } else if path_leaf.width() == 1 && query_leaf.width() == 1 {
                    self.on_mismatch(
                        matched,
                        path,
                        query,
                        key,
                    )
                } else {
                    // expand states to find matching prefix
                    NextStates::Prefixes(
                        key,
                        matched,
                        self.prefix_states(
                            root, 
                            path_leaf,
                            PathPair::GraphMajor(path.clone(), query.clone()),
                        )
                        .into_iter()
                        .chain(
                            self.prefix_states(
                                root,
                                query_leaf,
                                PathPair::QueryMajor(query, path),
                            )
                        )
                        .collect_vec()
                    )
                }
            Ordering::Greater =>
                // continue in prefix of child
                NextStates::Prefixes(
                    key,
                    matched,
                    self.prefix_states(
                        root,
                        path_leaf,
                        PathPair::GraphMajor(path, query),
                    )
                ),
            Ordering::Less =>
                NextStates::Prefixes(
                    key,
                    matched,
                    self.prefix_states(
                        root, 
                        query_leaf,
                        PathPair::QueryMajor(query, path),
                    )
                ),
        }
    }
    fn on_match(
        &mut self,
        mode: PathPairMode,
        mut path: R::Advanced,
        mut query: R::Query,
        key: CacheKey,
        root: CacheKey,
    ) -> NextStates<R> {
        //path.add_match_width::<_, D, _>(self.trav());
        if query.advance::<_, D, _>(self.trav()).is_continue() {
            if path.advance::<_, D, _>(self.trav()).is_continue() {
                NextStates::Child(
                    key,
                    true,
                    ChildState {
                        root,
                        paths: PathPair::from_mode(path, query, mode)
                    }
                )
            } else {
                self.next_parents(
                    R::Primer::from(path),
                    true,
                    key,
                    query
                )
            }
        } else {
            // query end
            //path.child_path_mut::<End>().simplify::<_, D, _>(self.trav());
            self.on_end(
                true,
                path,
                query,
                key,
                RangeKind::QueryEnd
            )
        }
    }
    fn on_end(
        &mut self,
        matched: bool,
        path: R::Advanced,
        query: R::Query,
        key: CacheKey,
        range_kind: RangeKind,
    ) -> NextStates<R> {
        let root_key = path.root_key();

        NextStates::End(
            key,
            matched,
            EndState {
                root: root_key,
                query,
                kind:
                    if path.raw_child_path::<End>().is_empty()
                        && path.child_pos::<Start>() == path.child_pos::<End>()
                    {
                        todo!("handle this")
                        //EndKind::Postfix(PostfixEnd {
                        //    path: path.pop_path::<_, D, _>(self.trav()),
                        //})
                    } else {
                        EndKind::Range(RangeEnd {
                            kind: range_kind,
                            entry: path.role_leaf_child_location::<End>().unwrap(),
                            path,
                        })
                    }
            }
        )
    }
    fn on_mismatch(
        &mut self,
        matched: bool,
        mut path: R::Advanced,
        query: R::Query,
        key: CacheKey,
    ) -> NextStates<R> {
        path.retract::<_, D, _, R>(self.trav());
        self.on_end(
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
        root: CacheKey,
        index: Child,
        paths: FolderPathPair<R>,
    ) -> Vec<ChildState<R>> {
        self.trav().graph()
            .expect_vertex_data(index)
            .get_child_patterns().iter()
            .sorted_unstable_by_key(|(_, p)| p.first().unwrap().width)
            .map(|(&pid, child_pattern)| {
                let sub_index = D::head_index(child_pattern.borrow());
                let mut paths = paths.clone();
                paths.push_major(ChildLocation::new(index, pid, sub_index));
                ChildState {
                    root,
                    paths,
                }
            })
            .collect_vec()
    }
    /// states generated from a query start state
    /// (query end or start parent states)
    fn query_start(
        &self,
        key: CacheKey,
        mut state: StartState<R>,
    ) -> NextStates<R> {
        if state.query.advance::<_, D, _>(self.trav()).is_continue() {
            NextStates::Parents(
                key,
                false,
                S::gen_parent_states(
                    self.trav(),
                    &state.query,
                    state.index,
                    |p, trav|
                        <_ as IntoPrimer<R>>::into_primer(
                            MatchEnd::Complete(state.index),
                            p,
                        )
                )
            )
        } else {
            todo!("query end");
            NextStates::Empty
        }
    }
}