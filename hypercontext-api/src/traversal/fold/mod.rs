use super::{
    cache::{
        key::{
            directed::up::UpKey,
            props::RootKey,
        },
        TraversalCache,
    },
    result::FoundRange,
    state::{
        bottom_up::{
            start::StartState,
            BUNext,
        },
        cursor::PatternRangeCursor,
        top_down::end::{
            EndKind,
            EndState,
        },
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
use std::fmt::Debug;
use transition::TransitionIter;

pub mod foldable;
pub mod state;
pub mod states;
pub(crate) mod transition;
use crate::{
    graph::vertex::wide::Wide,
    path::structs::query_range_path::FoldablePath,
    traversal::split::trace::{
        context::TraceContext,
        traceable::Traceable,
    },
};

pub trait MakeStartState {
    fn start_state(&self) -> StartState;
}
impl<K: TraversalKind> MakeStartState for CursorInit<K> {
    fn start_state(&self) -> StartState {
        let start_index = self.cursor.path.start_index(&self.trav);

        StartState {
            index: start_index,
            key: UpKey::new(start_index, 0.into()),
            cursor: self.cursor.clone(),
        }
    }
}
pub struct CursorInit<K: TraversalKind> {
    pub trav: K::Trav,
    pub cursor: PatternRangeCursor,
}

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
        let (parents, end_state) = match start.next_states::<K>(&trav) {
            BUNext::Parents(p) => (FromIterator::from_iter(p.inner), None),
            BUNext::End(end) => (Default::default(), Some(end.inner)),
        };
        let start_index = cursor.path.start_index(&trav);
        Self {
            tctx: TraversalContext {
                parents,
                children: Default::default(),
                end: Default::default(),
                cache: TraversalCache::new(&trav, start_index),
                trav,
            },
            max_width: start_index.width(),
            start_index,
            end_state,
        }
    }
}

impl<'a, K: TraversalKind> FoldContext<K> {
    fn fold(mut self) -> Result<FinishedState, ErrorState> {
        TransitionIter { fctx: &mut self }.for_each(|_| {});
        self.finish_fold()
    }
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
