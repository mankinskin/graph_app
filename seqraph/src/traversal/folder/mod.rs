use crate::*;
use super::*;
pub mod state;
pub use state::*;

pub type Folder<Ty>
    = <Ty as DirectedTraversalPolicy>::Trav;

pub trait TraversalFolder<
    S: DirectedTraversalPolicy<Trav=Self>,
>: Sized + Traversable {
    type NodeCollection: NodeCollection;

    //fn map_state(
    //    &self,
    //    acc: ControlFlow<Self::Break, Self::Continue>,
    //    node: TraversalState<R, Q>
    //) -> ControlFlow<Self::Break, Self::Continue>;

    //#[instrument(skip(self))]
    fn fold_query<P: IntoPattern>(
        &self,
        query_pattern: P,
    ) -> Result<TraversalResult, (NoMatch, QueryRangePath)> {
        let query_pattern = query_pattern.into_pattern();
        debug!("fold {:?}", query_pattern);
        let query = QueryState::new::<<Self::Kind as GraphKind>::Direction, _>(
            query_pattern.borrow() as &[Child]
        )
        .map_err(|(err, q)| (err, q.to_rooted(query_pattern.clone())))?;
        let start_index = query.clone().to_rooted(query_pattern.clone()).role_leaf_child::<End, _>(self);

        let start = StartState {
            index: start_index,
            query: query.clone(),
        };
        let mut states = OrderedTraverser::<_, _, Self::NodeCollection>::new(self);
        let (start_key, mut cache) = TraversalCache::new(&start, query_pattern.clone());
        states.extend(
            states.query_start(start_key, query.clone().to_cached(&mut cache), start)
                .into_states()
                .into_iter()
                .map(|n| (1, n))
        );
        let mut end_states = vec![];
        let mut max_width = 0;

        // 1. expand first parents
        // 2. expand next children/parents

        while let Some((depth, next_states)) = states.next_states(&mut cache) {
            match next_states {
                NextStates::End(next) => {
                    let state = next.inner;
                    debug!("{:#?}", state);
                    let width = state.width();
                    if width > start_index.width() && width >= max_width {
                        match &state.kind {
                            EndKind::Range(p) => {
                                let root_entry = p.path.role_root_child_location::<Start>().sub_index;
                                cache.add_path(
                                    self,
                                    root_entry,
                                    &p.path,
                                    state.root_pos,
                                    true,
                                )
                            },
                            EndKind::Prefix(p) =>
                                cache.add_path(
                                    self,
                                    0,
                                    &p.path,
                                    state.root_pos,
                                    true,
                                ),
                            _ => {}
                        }
                        // stop other paths not with this root
                        //if !matches!(state.kind, EndKind::Complete(_)) && cache.expect(&state.root_key()).num_bu_edges() < 2 {
                        //}
                        if let Some(root_key) = state.waiting_root_key() {
                            // this must happen before simplification
                            states.extend(
                                cache.continue_waiting(&root_key)
                            );
                        }

                        if width > max_width {
                            max_width = width;
                            end_states.clear();
                        }
                        let is_final = state.reason == EndReason::QueryEnd && matches!(state.kind, EndKind::Complete(_));
                        end_states.push(state);
                        if is_final {
                            break;
                        }
                    } else {
                        // stop other paths with this root
                        states.prune_below(state.root_key());
                    }
                },
                _ => {
                    states.extend(
                        next_states.into_states()
                            .into_iter()
                            .map(|nstate| (depth, nstate))
                    );
                },
            }
        }
        debug!("end roots: {:#?}", end_states.iter()
            .map(|s| {
                let root = s.root_parent();
                (root.index(), root.width(), s.root_pos.pos)
            }).collect_vec()
        );
        Ok(if end_states.is_empty() {
            TraversalResult {
                query: query.to_rooted(query_pattern),
                result: FoldResult::Complete(start_index)
            }
        } else {
            let final_states = end_states.iter()
                .map(|state|
                    FinalState {
                        num_parents: cache
                            .get(&state.root_key())
                            .unwrap()
                            .num_parents(),
                        state,
                    }
                )
                .sorted() 
                .collect_vec();
            let fin = final_states.first().unwrap();
            let query = fin.state.query.clone();
            let found_path = if let EndKind::Complete(c) = &fin.state.kind {
                    FoldResult::Complete(*c)
                } else {
                    // todo: complete bottom edges of root if 
                    // assert same root
                    let min_end = end_states.iter()
                        .min_by(|a, b| a.root_key().index.width().cmp(&b.root_key().index.width()))
                        .unwrap();
                    let root = min_end.root_key().index;
                    let end_pos = min_end.width();
                    let state = FoldState {
                        cache,
                        root,
                        end_pos: end_pos.into(),
                        end_states,
                        start: start_key.index,
                    };
                    //state.trace_subgraph(self);
                    FoldResult::Incomplete(state)
                };
            TraversalResult {
                query: query.to_rooted(query_pattern),
                result: found_path,
            }
        })
    }
}
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct FinalState<'a> {
    pub num_parents: usize,
    pub state: &'a EndState,
}
impl PartialOrd for FinalState<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for FinalState<'_> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.num_parents.cmp(&other.num_parents)
            .then_with(||
                other.state.is_complete().cmp(&self.state.is_complete())
                    .then_with(||
                        other.state.root_key().width().cmp(&self.state.root_key().width())
                    )
            )
    }
}