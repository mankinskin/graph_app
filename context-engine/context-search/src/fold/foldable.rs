use context_trace::{
    graph::getters::ErrorReason,
    path::structs::{
        query_range_path::FoldablePath,
        rooted::{
            pattern_range::PatternRangePath,
            role_path::PatternEndPath,
        },
    },
};

use crate::{
    fold::{
        result::{
            FinishedKind,
            FinishedState,
        },
        IntoFoldCtx,
    },
    traversal::{
        state::{
            cursor::{
                PatternCursor,
                PatternRangeCursor,
                ToCursor,
            },
            start::StartCtx,
        },
        TraversalKind,
    },
};
use context_trace::graph::vertex::pattern::Pattern;
use std::fmt::Debug;

use super::FoldCtx;

pub type FoldResult = Result<FinishedState, ErrorState>;

#[derive(Debug)]
pub struct ErrorState {
    pub reason: ErrorReason,
    pub found: Option<FinishedKind>,
}
impl From<ErrorReason> for ErrorState {
    fn from(reason: ErrorReason) -> Self {
        Self {
            reason,
            found: None,
        }
    }
}

pub trait Foldable: Sized {
    fn to_fold_context<K: TraversalKind>(
        self,
        trav: K::Trav,
    ) -> Result<FoldCtx<K>, ErrorState>;

    fn fold<K: TraversalKind>(
        self,
        trav: K::Trav,
    ) -> Result<FinishedState, ErrorState> {
        self.to_fold_context::<K>(trav).and_then(|ctx| ctx.fold())
    }
}

impl Foldable for &'_ Pattern {
    fn to_fold_context<K: TraversalKind>(
        self,
        trav: K::Trav,
    ) -> Result<FoldCtx<K>, ErrorState> {
        PatternRangePath::from(self).to_fold_context::<K>(trav)
    }
}
impl Foldable for Pattern {
    fn to_fold_context<K: TraversalKind>(
        self,
        trav: K::Trav,
    ) -> Result<FoldCtx<K>, ErrorState> {
        PatternRangePath::from(self).to_fold_context::<K>(trav)
    }
}

impl Foldable for PatternEndPath {
    fn to_fold_context<K: TraversalKind>(
        self,
        trav: K::Trav,
    ) -> Result<FoldCtx<K>, ErrorState> {
        self.to_range_path()
            .to_cursor(&trav)
            .to_fold_context::<K>(trav)
    }
}
impl Foldable for PatternRangePath {
    fn to_fold_context<K: TraversalKind>(
        self,
        trav: K::Trav,
    ) -> Result<FoldCtx<K>, ErrorState> {
        self.to_range_path()
            .to_cursor(&trav)
            .to_fold_context::<K>(trav)
    }
}
impl Foldable for PatternRangeCursor {
    fn to_fold_context<K: TraversalKind>(
        self,
        trav: K::Trav,
    ) -> Result<FoldCtx<K>, ErrorState> {
        PatternCursor::from(self).to_fold_context(trav)
    }
}

impl Foldable for PatternCursor {
    fn to_fold_context<K: TraversalKind>(
        self,
        trav: K::Trav,
    ) -> Result<FoldCtx<K>, ErrorState> {
        let start_index = self.path.start_index(&trav);
        StartCtx {
            index: start_index,
            cursor: self,
            trav,
        }
        .into_fold_context()
    }
}
