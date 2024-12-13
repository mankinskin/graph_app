use crate::{graph::getters::NoMatch,
    path::{
        accessors::role::End,
        structs::query_range_path::QueryRangePath,
    },
    traversal::{
        cache::{
            key::{
                root::RootKey, DirectedKey
            },
            state::{
                end::{
                    EndKind,
                    EndReason,
                }, query::QueryState, NextStates, StateNext
            },
            TraversalCache,
        },
        context::{
            QueryContext,
            TraversalContext,
        },
        folder::state::{
            FinalState, FoldResult, FoldState
        },
        iterator::{
            traverser::{
                pruning::PruneStates, ExtendStates
            }, TraversalIterator
        },
        result::{
            kind::RoleChildPath, TraversalResult
        },
        traversable::Traversable,
    },
};
use std::borrow::Borrow;
use crate::graph::vertex::{
    pattern::IntoPattern,
    wide::Wide,
};

use super::trace::Trace;

pub mod state;
pub struct FoldFinished {
    end_state: EndState,
    cache: TraversalCache,
}
impl FoldFinished {
    pub fn to_traversal_result(self) -> FoldResult {
        let final_state = FinalState {
            num_parents: self.cache
                .get(&DirectedKey::from(self.end_state.root_key()))
                .unwrap()
                .num_parents(),
            state: &self.end_state,
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
    }
}
pub trait TraversalFolder: Sized + Traversable {
    type Iterator<'a>: TraversalIterator<'a, Trav = Self> + From<&'a Self>
    where
        Self: 'a;

    //#[instrument(skip(self))]
    fn fold_pattern<P: IntoPattern>(
        &self,
        query_pattern: P,
    ) -> Result<TraversalResult, (NoMatch, QueryRangePath)>
    {
        let query_pattern = query_pattern.into_pattern();
        //debug!("fold {:?}", query_pattern);
        let query = QueryState::new::<Self::Kind, _>(query_pattern.borrow())
            .map_err(|(err, q)| (err, q.to_rooted(query_pattern.clone())))?;
        let query_range_path = query
            .clone()
            .to_rooted(query_pattern.clone());
        let query_root = QueryContext::new(query_pattern);
        self.fold_query(query_root, query_range_path, query)
    }
    //#[instrument(skip(self))]
    fn fold_query(
        &self,
        query_root: QueryContext,
        query_range_path: QueryRangePath,
        query: QueryState,
    ) -> Result<TraversalResult, (NoMatch, QueryRangePath)>
    {
        let start_index = query_range_path
            .role_leaf_child::<End, _>(self);

        //let query_ctx = QueryContext::new(query_pattern.clone());

        let (mut states, mut cache) =
            TraversalCache::new(self, start_index, &query_root, query.clone());

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
                    NextStates::Child(_) | NextStates::Prefixes(_) | NextStates::Parents(_) => {
                        states.extend(
                            next_states
                                .into_states()
                                .into_iter()
                                .map(|nstate| (depth + 1, nstate)),
                        );
                    }
                    NextStates::Empty => {}
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
        end_state.map(|state|
            state.map(|state|
                FoldFinished {
                    end_state: state,
                    cache,
                }.to_traversal_result()
            ).or_else(||
                TraversalResult {
                    //query: query.to_rooted(query_root.query_root),
                    query: query_range_path,
                    result: FoldResult::Complete(start_index),
                }
            )
        )
    }
}
