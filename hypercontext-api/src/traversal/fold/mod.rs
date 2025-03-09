use super::{
    cache::key::{
        directed::DirectedKey,
        props::RootKey,
    },
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
use state::{
    FinalState,
    FoldState,
};
use std::fmt::Debug;

pub mod foldable;
pub mod state;
pub mod states;
pub(crate) mod transition;
use crate::traversal::state::top_down::trace::{
    TraceContext,
    TraceInit,
};

/// context for running fold traversal
#[derive(Debug)]
pub struct FoldContext<K: TraversalKind> {
    pub tctx: TraversalContext<K>,
    pub start_index: Child,

    pub max_width: usize,
    pub end_state: Option<EndState>,
}

impl<'a, K: TraversalKind> FoldContext<K> {
    fn finish_fold(mut self) -> Result<FinishedState, ErrorState> {
        if let Some(state) = self.end_state {
            state.trace(&mut TraceContext {
                cache: &mut self.tctx.cache,
                trav: &self.tctx.trav,
            });
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
                    cache: self.tctx.cache,
                    root,
                    end_state: state,
                    start: self.start_index,
                })
            };
            Ok(FinishedState { result: found_path })
        } else {
            Err(ErrorState {
                reason: ErrorReason::NotFound,
                found: Some(FoundRange::Complete(self.start_index)),
            })
        }
    }
}
