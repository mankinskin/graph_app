use super::{
    result::FinishedKind,
    state::{
        cursor::PatternRangeCursor,
        start::StartCtx,
    },
    TraversalContext,
    TraversalKind,
};
use crate::traversal::result::FinishedState;
use context_trace::{
    graph::vertex::{
        child::Child,
        wide::Wide,
    },
    trace::{
        cache::key::props::RootKey,
        traceable::Traceable,
        TraceContext,
    },
};
use foldable::ErrorState;
use std::fmt::Debug;

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
        })
    }
}
/// context for running fold traversal
#[derive(Debug)]
pub struct FoldContext<K: TraversalKind> {
    pub tctx: TraversalContext<K>,
    pub start_index: Child,
    pub max_width: usize,
}

impl<K: TraversalKind> Iterator for FoldContext<K> {
    type Item = ();

    fn next(&mut self) -> Option<Self::Item> {
        self.tctx.next()
    }
}

impl<'a, K: TraversalKind> FoldContext<K> {
    fn fold(mut self) -> Result<FinishedState, ErrorState> {
        (&mut self).for_each(|_| ());
        let end = self.tctx.last_end;
        {
            let mut ctx = TraceContext {
                trav: &self.tctx.trav,
                cache: &mut self.tctx.cache,
            };
            end.clone().trace(&mut ctx);
        }
        Ok(FinishedState {
            cache: self.tctx.cache,
            root: end.root_key().index,
            start: self.start_index,
            kind: FinishedKind::from(end),
        })
        //Err(ErrorState {
        //    reason: ErrorReason::SingleIndex(self.start_index),
        //    found: Some(FinishedKind::Complete(self.start_index)),
        //},
    }
}
