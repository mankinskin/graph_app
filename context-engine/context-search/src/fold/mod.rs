use crate::{
    fold::result::{
        FinishedKind,
        FinishedState,
    },
    traversal::{
        IntoTraversalCtx,
        TraversalCtx,
        TraversalKind,
    },
};
use context_trace::{
    graph::{
        getters::IndexWithPath,
        vertex::{
            child::Child,
            has_vertex_index::ToChild,
            wide::Wide,
        },
    },
    trace::{
        cache::key::props::RootKey,
        traceable::Traceable,
    },
};
use foldable::ErrorState;
use std::fmt::Debug;

pub mod foldable;
pub mod result;
pub mod state;

pub trait IntoFoldCtx<K: TraversalKind> {
    fn into_fold_context(self) -> Result<FoldCtx<K>, ErrorState>;
}

impl<K: TraversalKind, S: IntoTraversalCtx<K> + ToChild> IntoFoldCtx<K> for S {
    fn into_fold_context(self) -> Result<FoldCtx<K>, ErrorState> {
        let start_index = self.to_child();
        self.into_traversal_context().map(|tctx| FoldCtx {
            tctx,
            max_width: start_index.width(),
            start_index,
        })
    }
}
/// context for running fold traversal
#[derive(Debug)]
pub struct FoldCtx<K: TraversalKind> {
    pub tctx: TraversalCtx<K>,
    pub start_index: Child,
    pub max_width: usize,
}

impl<K: TraversalKind> Iterator for FoldCtx<K> {
    type Item = ();
    fn next(&mut self) -> Option<Self::Item> {
        self.tctx.next()
    }
}

impl<K: TraversalKind> FoldCtx<K> {
    fn fold(mut self) -> Result<FinishedState, ErrorState> {
        (&mut self).for_each(|_| ());
        let end = self.tctx.last_match;
        end.trace(&mut self.tctx.match_iter.0);
        Ok(FinishedState {
            cache: self.tctx.match_iter.0.cache,
            root: IndexWithPath {
                index: end.root_key().index,
                path: end.cursor.path.clone().into(),
            },
            start: self.start_index,
            kind: FinishedKind::from(end),
        })
        //Err(ErrorState {
        //    reason: ErrorReason::SingleIndex(self.start_index),
        //    found: Some(FinishedKind::Complete(self.start_index)),
        //},
    }
}
