use crate::{
    graph::{
        getters::ErrorReason,
        vertex::pattern::IntoPattern,
    },
    path::structs::{
        query_range_path::FoldablePath,
        rooted::{
            pattern_range::PatternRangePath,
            role_path::PatternEndPath,
        },
    },
    traversal::{
        result::{
            FinishedKind,
            FinishedState,
        },
        state::{
            cursor::{
                PatternRangeCursor,
                ToCursor,
            },
            start::StartCtx,
        },
        TraversalKind,
    },
};
use std::fmt::Debug;

use super::FoldContext;

pub type FoldResult = Result<FinishedState, ErrorState>;

#[derive(Debug)]
pub struct ErrorState {
    pub reason: ErrorReason,
    pub found: Option<FinishedKind>,
}

pub trait Foldable: Sized {
    fn to_fold_context<K: TraversalKind>(
        self,
        trav: K::Trav,
    ) -> Result<FoldContext<K>, ErrorState>;

    fn fold<K: TraversalKind>(
        self,
        trav: K::Trav,
    ) -> Result<FinishedState, ErrorState> {
        self.to_fold_context::<K>(trav).and_then(|ctx| ctx.fold())
    }
}

impl<P: IntoPattern> Foldable for P {
    fn to_fold_context<K: TraversalKind>(
        self,
        trav: K::Trav,
    ) -> Result<FoldContext<K>, ErrorState> {
        // build cursor path
        let path = PatternRangePath::from(self.into_pattern());
        path.to_fold_context::<K>(trav)
    }
}

impl Foldable for PatternEndPath {
    fn to_fold_context<K: TraversalKind>(
        self,
        trav: K::Trav,
    ) -> Result<FoldContext<K>, ErrorState> {
        self.to_range_path().to_cursor().to_fold_context::<K>(trav)
    }
}
impl Foldable for PatternRangePath {
    fn to_fold_context<K: TraversalKind>(
        self,
        trav: K::Trav,
    ) -> Result<FoldContext<K>, ErrorState> {
        self.to_range_path().to_cursor().to_fold_context::<K>(trav)
    }
}
impl Foldable for PatternRangeCursor {
    fn to_fold_context<K: TraversalKind>(
        self,
        trav: K::Trav,
    ) -> Result<FoldContext<K>, ErrorState> {
        let start_index = self.path.start_index(&trav);
        TryFrom::try_from(StartCtx {
            index: start_index,
            cursor: self,
            trav,
        })
    }
}
