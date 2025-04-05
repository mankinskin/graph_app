use super::{
    cache::key::{
        directed::up::UpKey,
        props::RootKey,
    },
    result::FoundRange,
    state::{
        bottom_up::start::{
            StartContext,
            StartState,
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
        vertex::{
            child::Child,
            wide::Wide,
        },
    },
    path::structs::query_range_path::FoldablePath,
    traversal::{
        result::FinishedState,
        split::trace::{
            context::TraceContext,
            traceable::Traceable,
        },
    },
};
use foldable::ErrorState;
use state::FoldState;
use std::{
    fmt::Debug,
    ops::ControlFlow::Break,
};

pub mod foldable;
pub mod state;
pub mod states;

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

impl<K: TraversalKind> TryFrom<StartContext<K>> for FoldContext<K> {
    type Error = ErrorState;
    fn try_from(start: StartContext<K>) -> Result<Self, Self::Error> {
        let start_index = start.state.index.clone();
        TraversalContext::try_from(start).map(|tctx| Self {
            tctx,
            max_width: start_index.width(),
            start_index,
        })
    }
}
/// context for running fold traversal
#[derive(Debug)]
pub struct FoldContext<K: TraversalKind> {
    pub tctx: TraversalContext<K>,
    pub start_index: Child,
    pub max_width: usize,
    //pub end_state: Option<EndState>,
}

impl<K: TraversalKind> TryFrom<CursorInit<K>> for FoldContext<K> {
    type Error = <Self as TryFrom<StartContext<K>>>::Error;
    fn try_from(init: CursorInit<K>) -> Result<Self, Self::Error> {
        let start = init.start_state();
        TryFrom::try_from(StartContext {
            trav: init.trav,
            state: start,
        })
    }
}
impl<K: TraversalKind> Iterator for FoldContext<K> {
    type Item = Option<EndState>;

    fn next(&mut self) -> Option<Self::Item> {
        self.tctx.next().and_then(|end_state| {
            if let Break(end) = end_state {
                assert!(
                    end.width() >= self.max_width,
                    "Parents not evaluated in order"
                );
                if end.width() > self.max_width {
                    self.max_width = end.width();
                    //self.end_state = Some(end);
                }
                Some(end.is_final().then_some(end))
            } else {
                Some(None)
            }
        })
    }
}

impl<'a, K: TraversalKind> FoldContext<K> {
    fn fold(mut self) -> Result<FinishedState, ErrorState> {
        (&mut self)
            .find_map(|end| end)
            .map(|state| {
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
            })
            .unwrap_or_else(|| {
                Err(ErrorState {
                    reason: ErrorReason::SingleIndex(self.start_index),
                    found: Some(FoundRange::Complete(self.start_index)),
                })
            })
    }
}
