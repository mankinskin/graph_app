use super::{
    cache::key::{
        directed::DirectedKey,
        props::RootKey,
    },
    result::FoundRange,
    state::{
        cursor::{
            RangeCursor,
            ToCursor,
        },
        next_states::NextStates,
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
            pattern::{
                IntoPattern,
                Pattern,
            },
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
        fold::{
            apply::ApplyStatesCtx,
            state::{
                FinalState,
                FoldState,
            },
        },
        result::FinishedState,
    },
};
use init::{
    CursorInit,
    InitStates,
};
use std::fmt::Debug;

pub(crate) mod apply;
pub(crate) mod init;
pub mod state;

#[derive(Debug)]
pub struct ErrorState {
    pub reason: ErrorReason,
    //pub query: QueryState,
    pub found: Option<FoundRange>,
}

pub type FoldResult = Result<FinishedState, ErrorState>;

pub trait Foldable {
    fn fold<K: TraversalKind>(
        self,
        trav: K::Trav,
    ) -> FoldResult;
}

#[macro_export]
macro_rules! impl_foldable {
    ($(< $( $lt:tt )+>)? $t:ty, $f:ident) => {
        impl $(< $lt >)? Foldable for $t {
            fn fold<K: TraversalKind>(
                self,
                trav: K::Trav,
            ) -> FoldResult {
                FoldContext::<K>::$f(trav, self)
            }
        }
    };
}
//impl_foldable!(QueryState, fold_query);
impl_foldable!(PatternRangePath, fold_path);
impl_foldable!(Pattern, fold_pattern);
impl_foldable!(&'_ Pattern, fold_pattern);
//impl_foldable!([Child; 2], fold_pattern);
impl_foldable!(PatternEndPath, fold_path);

/// context for running fold traversal
#[derive(Debug)]
pub struct FoldContext<K: TraversalKind> {
    pub tctx: TraversalContext<K>,
    pub start_index: Child,

    pub max_width: usize,
    pub end_state: Option<EndState>,
}
impl<K: TraversalKind> Iterator for TraversalContext<K> {
    type Item = (usize, NextStates);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some((depth, tstate)) = self.states.next() {
            self.traversal_next_states(tstate)
                .map(|states| (depth, states))
        } else {
            None
        }
    }
}
impl<'a, K: TraversalKind> FoldContext<K> {
    pub fn fold_pattern<P: IntoPattern>(
        trav: K::Trav,
        pattern: P,
    ) -> FoldResult {
        // build cursor path
        let path = PatternRangePath::from(pattern.into_pattern());
        Self::fold_path(trav, path)
    }
    pub fn fold_path(
        trav: K::Trav,
        query: impl FoldablePath,
    ) -> Result<FinishedState, ErrorState> {
        Self::fold_cursor(trav, query.to_range_path().to_cursor())
    }
    pub fn fold_cursor(
        trav: K::Trav,
        cursor: RangeCursor,
    ) -> Result<FinishedState, ErrorState> {
        let init = CursorInit::<K> {
            trav: &trav,
            cursor: &cursor,
        };
        let start_index = init.cursor.path.start_index(&trav);

        let mut ctx = Self {
            tctx: TraversalContext {
                states: init.init_context(),
                trav,
            },
            end_state: None,
            max_width: start_index.width(),
            start_index,
        };
        ctx.fold_states();
        ctx.finish_fold(cursor.path)
    }
    fn fold_states(&mut self) {
        ApplyStatesCtx { fctx: self }.for_each(|_| {})
    }
    fn finish_fold(
        self,
        path: PatternRangePath,
    ) -> Result<FinishedState, ErrorState> {
        if let Some(state) = self.end_state {
            Ok(FoldFinished {
                end_state: state,
                cache: self.tctx.states.cache,
                start_index: self.start_index,
                query_root: path.root,
            }
            .to_traversal_result())
        } else {
            Err(ErrorState {
                reason: ErrorReason::NotFound,
                found: Some(FoundRange::Complete(self.start_index, path)),
            })
        }
    }
}
pub struct FoldFinished {
    pub end_state: EndState,
    pub cache: TraversalCache,
    pub start_index: Child,
    pub query_root: Pattern,
}
impl FoldFinished {
    pub fn to_traversal_result(self) -> FinishedState {
        let final_state = FinalState {
            num_parents: self
                .cache
                .get(&DirectedKey::from(self.end_state.root_key()))
                .unwrap()
                .num_parents(),
            state: &self.end_state,
        };
        let cursor = final_state.state.cursor.clone();
        let found_path = if let EndKind::Complete(c) = &final_state.state.kind {
            FoundRange::Complete(*c, cursor.path)
        } else {
            // todo: complete bottom edges of root if
            // assert same root
            //let min_end = end_states.iter()
            //    .min_by(|a, b| a.root_key().index.width().cmp(&b.root_key().index.width()))
            //    .unwrap();
            let root = self.end_state.root_key().index;
            let state = FoldState {
                cache: self.cache,
                root,
                end_state: self.end_state,
                start: self.start_index,
            };
            FoundRange::Incomplete(state)
        };
        FinishedState { result: found_path }
    }
}
