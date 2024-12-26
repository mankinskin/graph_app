use crate::{
    graph::{
        getters::NoMatch,
        vertex::{
            child::Child,
            pattern::{
                IntoPattern, Pattern
            }, wide::Wide,
        },
    }, path::accessors::role::End, traversal::{
        cache::{
            key::{
                root::RootKey, DirectedKey
            },
            TraversalCache,
        }, fold::state::{
            FinalState, FoldResult, FoldState
        }, result::TraversalResult, state::{
            end::{
                EndKind,
                EndState,
            },
            query::QueryState,
            start::StartState,
        }, traversable::Traversable
    }
};
use std::{borrow::Borrow, ops::ControlFlow};
use super::{
    cache::key::UpKey, container::{extend::ExtendStates, pruning::{PruneStates, PruningMap, PruningState}, StateContainer},iterator::policy::DirectedTraversalPolicy, result::kind::RoleChildPath, state::{traversal::TraversalState, ApplyStatesCtx}, traversable::TravKind
};
use init::{InitStates, QueryStateInit};
use itertools::Itertools;
use std::fmt::Debug;

pub mod state;
pub mod init;

pub trait TraversalKind: Debug {
    type Trav: Traversable;
    type Container: StateContainer;
    type Policy: DirectedTraversalPolicy<Trav = Self::Trav>;
}
//  1. Input
//      - Pattern
//      - QueryState
//  2. Init
//      - Trav
//      - start index
//      - start states
//  3. Fold
//      - TraversalCache
//      - FoldStepState

/// context for generating next states
#[derive(Debug)]
pub struct TraversalContext<'a, K: TraversalKind> {
    pub states: &'a mut StatesContext<K>,
    pub trav: &'a K::Trav,
}

impl<K: TraversalKind> Unpin for TraversalContext<'_, K> {}

#[derive(Debug)]
pub struct StatesContext<K: TraversalKind> {
    pub cache: TraversalCache,
    pub pruning_map: PruningMap,
    pub states: K::Container,
}
impl<K: TraversalKind> PruneStates for StatesContext<K> {
    fn clear(&mut self) {
        self.states.clear();
    }
    fn pruning_map(&mut self) -> &mut PruningMap {
        &mut self.pruning_map
    }
}
impl<K: TraversalKind> ExtendStates for StatesContext<K>
{
    fn extend<
        It: DoubleEndedIterator + Iterator<Item = (usize, TraversalState)>,
        In: IntoIterator<Item = (usize, TraversalState), IntoIter = It>,
    >(
        &mut self,
        iter: In,
    ) {
        let states = iter
            .into_iter()
            .map(|(d, s)| {
                // count states per root
                self.pruning_map
                    .entry(s.root_key())
                    .and_modify(|ps| ps.count += 1)
                    .or_insert(PruningState {
                        count: 1,
                        prune: false,
                    });
                (d, s)
            })
            .collect_vec();
        self.states.extend(states)
    }
}
impl<K: TraversalKind> Iterator for StatesContext<K> {
    type Item = (usize, TraversalState);

    fn next(&mut self) -> Option<Self::Item> {
        for (d, s) in self.states.by_ref() {
            let e = self.pruning_map.get_mut(&s.root_key()).unwrap();
            e.count -= 1;
            let pass = !e.prune;
            if e.count == 0 {
                self.pruning_map.remove(&s.root_key());
            }
            if pass {
                return Some((d, s));
            }
        }
        None
    }
}
/// context for running fold traversal
pub struct FoldContext<'a, K: TraversalKind> {
    pub trav: &'a K::Trav,
    pub start_index: Child,

    pub max_width: usize,
    pub end_state: Option<EndState>,
    pub states: StatesContext<K>,
}
impl<K: TraversalKind> Iterator for FoldContext<'_, K> {
    type Item = (usize, TraversalState);

    fn next(&mut self) -> Option<Self::Item> {
        self.states.next()
    }
}
impl<K: TraversalKind> FoldContext<'_, K>
{
    fn fold_pattern<P: IntoPattern>(
        trav: &K::Trav,
        query_pattern: P,
    ) -> Result<TraversalResult, (NoMatch, TraversalResult)>
    {
        let query_pattern = query_pattern.into_pattern();

        // build cursor path
        let query = QueryState::new::<TravKind<K::Trav>, _>(query_pattern.borrow())?;

        Self::fold_query(trav, query)
    }
    fn fold_query(
        trav: &K::Trav,
        query: QueryState,
    ) -> Result<TraversalResult, (NoMatch, TraversalResult)>
    {
        let init = QueryStateInit::<K> {
            trav,
            query,
        };
        let start_index = init.start_index();

        let ctx = FoldContext::<K> {
            states: StatesContext {
                cache: TraversalCache::new(trav, start_index),
                states: init.init_states(),
                pruning_map: Default::default(),
            },
            trav,
            end_state: None,
            max_width: start_index.width(),
            start_index,
        };
        ctx.fold_states()
    }
    fn fold_states(mut self) -> Result<TraversalResult, (NoMatch, TraversalResult)> {

        while let Some((depth, tstate)) = self.next() {
            let mut ctx = TraversalContext::<K> {
                trav: self.trav,
                states: &mut self.states,
            };
            if Some(ControlFlow::Break(())) == tstate.next_states(&mut ctx)
                .map(|next_states| next_states.apply(
                    ApplyStatesCtx {
                        tctx: &mut ctx,
                        max_width: &mut self.max_width,
                        end_state: &mut self.end_state,
                        depth,
                    }
                ))
            {
                break;
            }
        }
        //debug!("end roots: {:#?}", end_states.iter()
        //    .map(|s| {
        //        let root = s.root_parent();
        //        (root.index(), root.width(), s.root_pos.0)
        //    }).collect_vec()
        //);
        self.finish_fold()
    }
    fn finish_fold(self) -> Result<TraversalResult, (NoMatch, TraversalResult)> {
        self.end_state
            .map(|state| {
                FoldFinished {
                    end_state: state,
                    cache: self.states.cache,
                    start_index: self.start_index,
                    query_root: self.query_root,
                }
                .to_traversal_result()
            })
            .ok_or_else(|| {
                (
                    NoMatch::NotFound,
                    TraversalResult {
                        //query: query.to_rooted(query_root.query_root),
                        query: query_range_path,
                        result: FoldResult::Complete(self.start_index),
                    },
                )
            })
    }
}
pub struct FoldFinished {
    pub end_state: EndState,
    pub cache: TraversalCache,
    pub start_index: Child,
    pub query_root: Pattern,
}
impl FoldFinished {
    pub fn to_traversal_result(self) -> TraversalResult {
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
            let root = self.end_state.root_key().index;
            let state = FoldState {
                cache: self.cache,
                root,
                end_state: self.end_state,
                start: self.start_index,
            };
            FoldResult::Incomplete(state)
        };
        TraversalResult {
            query: query,
            result: found_path,
        }
    }
}
