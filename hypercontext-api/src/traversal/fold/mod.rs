use super::{
    states::StatesContext,
    TraversalContext,
    result::FoundRange,
    state::{
        traversal::TraversalState,
        ApplyStatesCtx,
    },
    traversable::TravKind, TraversalKind,
};
use crate::{
    graph::{
        getters::ErrorReason,
        vertex::{
            child::Child,
            pattern::{
                IntoPattern,
                Pattern,
            },
            wide::Wide,
        },
    },
    traversal::{
        cache::{
            key::{
                root::RootKey,
                DirectedKey,
            },
            TraversalCache,
        },
        fold::state::{
            FinalState,
            FoldState,
        },
        result::FinishedState,
        state::{
            end::{
                EndKind,
                EndState,
            },
            query::QueryState,
        },
    },
};
use init::{
    InitStates,
    QueryStateInit,
};
use std::{
    borrow::Borrow,
    fmt::Debug,
};

pub mod init;
pub mod state;

#[derive(Debug)]
pub struct ErrorState {
    pub reason: ErrorReason,
    //pub query: QueryState,
    pub found: Option<FoundRange>,
}
/// context for running fold traversal
#[derive(Debug)]
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
impl<'a, K: TraversalKind> FoldContext<'a, K> {
    pub fn fold_pattern<P: IntoPattern>(
        trav: &'a K::Trav,
        query_pattern: P,
    ) -> Result<FinishedState, ErrorState> {
        let query_pattern = query_pattern.into_pattern();

        // build cursor path
        let query = QueryState::new::<TravKind<K::Trav>, _>(query_pattern.borrow())?;

        Self::fold_query(trav, query)
    }
    pub fn fold_query(
        trav: &'a K::Trav,
        query: QueryState,
    ) -> Result<FinishedState, ErrorState> {
        let init = QueryStateInit::<K> {
            trav,
            query: &query,
        };
        let start_index = init.start_index();

        let mut ctx = Self {
            states: init.init_context(),
            trav,
            end_state: None,
            max_width: start_index.width(),
            start_index,
        };
        ctx.fold_states()?;
        ctx.finish_fold(query)
    }
    fn fold_states(&mut self) -> Result<(), ErrorState> {
        while let Some((depth, tstate)) = self.next() {
            let mut ctx = TraversalContext::<K> {
                trav: self.trav,
                states: &mut self.states,
            };
            if let Some(next_states) = tstate.next_states(&mut ctx)
            {
                if (ApplyStatesCtx {
                    tctx: &mut ctx,
                    max_width: &mut self.max_width,
                    end_state: &mut self.end_state,
                    depth,
                })
                .apply_transition(next_states)
                .is_break()
                {
                    break;
                }
            }
        }
        Ok(())
    }
    fn finish_fold(
        self,
        query: QueryState,
    ) -> Result<FinishedState, ErrorState> {
        if let Some(state) = self.end_state {
            Ok(FoldFinished {
                end_state: state,
                cache: self.states.cache,
                start_index: self.start_index,
                query_root: query.path.root,
            }
            .to_traversal_result())
        } else {
            Err(ErrorState {
                reason: ErrorReason::NotFound,
                found: Some(FoundRange::Complete(self.start_index, query)),
            })
        }
    }
}
pub struct FoldFinished {
    pub end_state: EndState,
    pub cache: TraversalCache,
    pub start_index: Child,
    pub query_root: Pattern,
}
impl FoldFinished {
    pub fn to_traversal_result(self) -> FinishedState {
        let final_state = FinalState {
            num_parents: self
                .cache
                .get(&DirectedKey::from(self.end_state.root_key()))
                .unwrap()
                .num_parents(),
            state: &self.end_state,
        };
        let query = final_state.state.query.clone();
        let found_path = if let EndKind::Complete(c) = &final_state.state.kind {
            FoundRange::Complete(*c, query)
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
            FoundRange::Incomplete(state)
        };
        FinishedState {
            result: found_path,
        }
    }
}
