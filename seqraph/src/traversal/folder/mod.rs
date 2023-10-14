use crate::*;

pub mod state;
pub use state::*;

use super::cache::trace::Trace;

pub trait TraversalFolder: Sized + Traversable {
    type Iterator<'a>: TraversalIterator<'a, Trav=Self> + From<&'a Self> where Self: 'a;

    //#[instrument(skip(self))]
    fn fold_query<P: IntoPattern>(
        &self,
        query_pattern: P,
    ) -> Result<TraversalResult, (NoMatch, QueryRangePath)> {
        let query_pattern = query_pattern.into_pattern();
        //debug!("fold {:?}", query_pattern);
        let query = QueryState::new::<Self::Kind, _>(
                query_pattern.borrow() as &[Child]
            )
            .map_err(|(err, q)|
                (err, q.to_rooted(query_pattern.clone()))
            )?;
        let start_index = query.clone()
            .to_rooted(query_pattern.clone())
            .role_leaf_child::<End, _>(self);
        let query_root = QueryContext::new(query_pattern);

        //let query_ctx = QueryContext::new(query_pattern.clone());
        
        let (mut states, mut cache) = TraversalCache::new(
            self,
            start_index,
            &query_root,
            query.clone(),
        );

        let mut end_state = None;
        let mut max_width = start_index.width();

        // 1. expand first parents
        // 2. expand next children/parents

        while let Some((depth, tstate)) = states.next() {
            if let Some(next_states) = {
                let mut ctx = TraversalContext::new(&query_root, &mut cache, &mut states);
                tstate.next_states(&mut ctx)
            } {
                match next_states {
                    NextStates::Child(_) |
                    NextStates::Prefixes(_) |
                    NextStates::Parents(_) => {
                        states.extend(
                            next_states.into_states()
                                .into_iter()
                                .map(|nstate| (depth + 1, nstate))
                        );
                    },
                    NextStates::Empty => {},
                    NextStates::End(StateNext { inner: end, .. }) => {
                        //debug!("{:#?}", state);
                        if end.width() >= max_width {

                            end.trace(self, &mut cache);

                            // note: not really needed with completion
                            //if let Some(root_key) = end.waiting_root_key() {
                            //    // continue paths also arrived at this root
                            //    // this must happen before simplification
                            //    states.extend(
                            //        cache.continue_waiting(&root_key)
                            //    );
                            //}
                            if end.width() > max_width {
                                max_width = end.width();
                                //end_states.clear();
                            }
                            let is_final = end.reason == EndReason::QueryEnd
                                && matches!(end.kind, EndKind::Complete(_));
                            end_state = Some(end);
                            if is_final {
                                break;
                            }
                        } else {
                            // larger root already found
                            // stop other paths with this root
                            states.prune_below(end.root_key());
                        }
                    }
                }
            }
        }
        //debug!("end roots: {:#?}", end_states.iter()
        //    .map(|s| {
        //        let root = s.root_parent();
        //        (root.index(), root.width(), s.root_pos.0)
        //    }).collect_vec()
        //);
        Ok(if let Some(state) = end_state {
            let final_state = 
                FinalState {
                    num_parents: cache
                        .get(&DirectedKey::from(state.root_key()))
                        .unwrap()
                        .num_parents(),
                    state: &state,
                };
            let query = final_state.state.query.clone();
            let found_path = if let EndKind::Complete(c) = &final_state.state.kind {
                    FoldResult::Complete(*c)
                } else {
                    // todo: complete bottom edges of root if 
                    // assert same root
                    //let min_end = end_states.iter()
                    //    .min_by(|a, b| a.root_key().index.width().cmp(&b.root_key().index.width()))
                    //    .unwrap();
                    let root = state.root_key().index;
                    let state = FoldState {
                        cache,
                        root,
                        end_state: state,
                        start: start_index,
                    };
                    FoldResult::Incomplete(state)
                };
            TraversalResult {
                query: query.to_rooted(query_root.query_root),
                result: found_path,
            }
        } else {
            TraversalResult {
                query: query.to_rooted(query_root.query_root),
                result: FoldResult::Complete(start_index)
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