use super::{
    cache::key::props::RootKey,
    result::FoundRange,
    state::top_down::end::{
        EndKind,
        EndState,
    },
    TraversalContext,
    TraversalKind,
};
use crate::{
    graph::{
        getters::ErrorReason,
        vertex::child::Child,
    },
    traversal::result::FinishedState,
};
use foldable::ErrorState;
use state::FoldState;
use states::init::CursorInit;
use std::fmt::Debug;

pub mod foldable;
pub mod state;
pub mod states;
pub(crate) mod transition;
use crate::{
    graph::vertex::wide::Wide,
    path::structs::query_range_path::FoldablePath,
    traversal::{
        fold::states::init::MakeStartState,
        trace::{
            context::TraceContext,
            traceable::Traceable,
        },
        TraversalCache,
    },
};
/// context for running fold traversal
#[derive(Debug)]
pub struct FoldContext<K: TraversalKind> {
    pub tctx: TraversalContext<K>,
    pub start_index: Child,
    pub max_width: usize,
    pub end_state: Option<EndState>,
}

impl<K: TraversalKind> From<CursorInit<K>> for FoldContext<K> {
    fn from(init: CursorInit<K>) -> Self {
        let start = init.start_state();
        let CursorInit { trav, cursor } = init;
        let starters = start
            .next_states::<K>(&trav)
            .into_states()
            .into_iter()
            .map(|s| (1, s));

        let start_index = cursor.path.start_index(&trav);
        Self {
            tctx: TraversalContext {
                states: FromIterator::from_iter(starters),
                cache: TraversalCache::new(&trav, start_index),
                trav,
            },
            max_width: start_index.width(),
            start_index,
            end_state: None,
        }
    }
}

impl<'a, K: TraversalKind> FoldContext<K> {
    fn finish_fold(self) -> Result<FinishedState, ErrorState> {
        if let Some(state) = self.end_state {
            let mut ctx = TraceContext {
                cache: self.tctx.cache,
                trav: self.tctx.trav,
            };
            state.trace(&mut ctx);
            //let cursor = final_state.state.cursor.clone();
            let found_path = if let EndKind::Complete(c) = &state.kind {
                FoundRange::Complete(*c) // cursor.path
            } else {
                // todo: complete bottom edges of root if
                // assert same root
                //let min_end = end_states.iter()
                //    .min_by(|a, b| a.root_key().index.width().cmp(&b.root_key().index.width()))
                //    .unwrap();
                let root = state.root_key().index;
                FoundRange::Incomplete(FoldState {
                    cache: ctx.cache,
                    root,
                    end_state: state,
                    start: self.start_index,
                })
            };
            Ok(FinishedState { result: found_path })
        } else {
            Err(ErrorState {
                reason: ErrorReason::SingleIndex(self.start_index),
                found: Some(FoundRange::Complete(self.start_index)),
            })
        }
    }
}
