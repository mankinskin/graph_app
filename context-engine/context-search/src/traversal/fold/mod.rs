use super::{
    result::FinishedKind,
    state::{
        cursor::PatternRangeCursor,
        end::EndState,
        start::StartCtx,
    },
    TraversalContext,
    TraversalKind,
};
use crate::traversal::result::FinishedState;
use context_trace::{
    graph::{
        getters::ErrorReason,
        vertex::{
            child::Child,
            wide::Wide,
        },
    },
    trace::{
        cache::key::props::RootKey,
        traceable::Traceable,
        TraceContext,
    },
};
use foldable::ErrorState;
use std::{
    fmt::Debug,
    ops::ControlFlow::Break,
};

pub mod foldable;
pub mod state;

pub struct CursorInit<K: TraversalKind> {
    pub trav: K::Trav,
    pub cursor: PatternRangeCursor,
}

impl<K: TraversalKind> TryFrom<StartCtx<K>> for FoldContext<K> {
    type Error = ErrorState;
    fn try_from(start: StartCtx<K>) -> Result<Self, Self::Error> {
        let start_index = start.index;
        TraversalContext::try_from(start).map(|tctx| Self {
            tctx,
            max_width: start_index.width(),
            start_index: start_index,
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
            },
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
            },
            None => Err(ErrorState {
                reason: ErrorReason::SingleIndex(self.start_index),
                found: Some(FinishedKind::Complete(self.start_index)),
            }),
        }
    }
}
