use crate::{
    graph::{
        getters::ErrorReason,
        vertex::{
            pattern::IntoPattern,
            wide::Wide,
        },
    },
    path::structs::{
        query_range_path::FoldablePath,
        rooted::{
            pattern_range::PatternRangePath,
            role_path::PatternEndPath,
        },
    },
    traversal::{
        cache::TraversalCache,
        fold::states::init::CursorInit,
        result::{
            FinishedState,
            FoundRange,
        },
        state::cursor::{
            PatternRangeCursor,
            ToCursor,
        },
        TraversalContext,
        TraversalKind,
    },
};
use std::fmt::Debug;

use super::{
    transition::StateTransitionIter,
    FoldContext,
};

pub type FoldResult = Result<FinishedState, ErrorState>;

#[derive(Debug)]
pub struct ErrorState {
    pub reason: ErrorReason,
    //pub query: QueryState,
    pub found: Option<FoundRange>,
}

pub trait Foldable: Sized {
    fn to_fold_context<K: TraversalKind>(
        self,
        trav: K::Trav,
    ) -> FoldContext<K>;
    fn fold<K: TraversalKind>(
        self,
        trav: K::Trav,
    ) -> Result<FinishedState, ErrorState> {
        let mut ctx = self.to_fold_context::<K>(trav);
        StateTransitionIter { fctx: &mut ctx }.for_each(|_| {});
        ctx.finish_fold()
    }
}

impl<P: IntoPattern> Foldable for P {
    fn to_fold_context<K: TraversalKind>(
        self,
        trav: K::Trav,
    ) -> FoldContext<K> {
        // build cursor path
        let path = PatternRangePath::from(self.into_pattern());
        path.to_fold_context::<K>(trav)
    }
}

impl Foldable for PatternEndPath {
    fn to_fold_context<K: TraversalKind>(
        self,
        trav: K::Trav,
    ) -> FoldContext<K> {
        self.to_range_path().to_cursor().to_fold_context::<K>(trav)
    }
}
impl Foldable for PatternRangePath {
    fn to_fold_context<K: TraversalKind>(
        self,
        trav: K::Trav,
    ) -> FoldContext<K> {
        self.to_range_path().to_cursor().to_fold_context::<K>(trav)
    }
}
impl Foldable for PatternRangeCursor {
    fn to_fold_context<K: TraversalKind>(
        self,
        trav: K::Trav,
    ) -> FoldContext<K> {
        let start_index = self.path.start_index(&trav);

        let init = CursorInit::<K> {
            trav: &trav,
            cursor: &self,
        };
        FoldContext {
            tctx: TraversalContext {
                states: From::from(init),
                cache: TraversalCache::new(&trav, start_index),
                trav,
            },
            max_width: start_index.width(),
            start_index,
            end_state: None,
        }
    }
}
