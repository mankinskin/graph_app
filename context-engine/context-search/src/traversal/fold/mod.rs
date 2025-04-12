use super::{
    cache::key::{
        directed::up::UpKey,
        props::RootKey,
    },
    result::FinishedKind,
    state::{
        bottom_up::start::{
            StartContext,
            StartState,
        },
        cursor::PatternRangeCursor,
        top_down::end::EndState,
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
    trace::{
        traceable::Traceable,
        TraceContext,
    },
    traversal::result::FinishedState,
};
use foldable::ErrorState;
use std::{
    fmt::Debug,
    ops::ControlFlow::Break,
};

pub mod foldable;
pub mod state;

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
            end_state: None,
        })
    }
}
/// context for running fold traversal
#[derive(Debug)]
pub struct FoldContext<K: TraversalKind> {
    pub tctx: TraversalContext<K>,
    pub start_index: Child,
    pub max_width: usize,
    pub end_state: Option<EndState>,
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
    type Item = ();

    fn next(&mut self) -> Option<Self::Item> {
        match self.tctx.next() {
            Some(Break(end)) => {
                assert!(
                    end.width() >= self.max_width,
                    "Parents not evaluated in order"
                );
                let is_final = end.is_final();
                if end.width() > self.max_width {
                    self.max_width = end.width();
                    self.end_state = Some(end);
                }
                (!is_final).then_some(())
            }
            Some(_) => Some(()),
            None => None,
        }
    }
}

impl<'a, K: TraversalKind> FoldContext<K> {
    fn fold(mut self) -> Result<FinishedState, ErrorState> {
        (&mut self).for_each(|_| ());
        match self.end_state {
            Some(end) => {
                let mut ctx = TraceContext {
                    cache: self.tctx.cache,
                    trav: self.tctx.trav,
                };
                end.trace(&mut ctx);
                Ok(FinishedState {
                    cache: ctx.cache,
                    root: end.root_key().index,
                    start: self.start_index,
                    kind: FinishedKind::from(end),
                })
            }
            None => Err(ErrorState {
                reason: ErrorReason::SingleIndex(self.start_index),
                found: Some(FinishedKind::Complete(self.start_index)),
            }),
        }
    }
}
