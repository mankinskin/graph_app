use super::{
    result::FoundRange,
    state::{
        cursor::{
            RangeCursor,
            ToCursor,
        },
        traversal::TraversalState,
        ApplyStatesCtx,
    },
    states::StatesContext,
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
            pattern_prefix::PatternPrefixPath,
            pattern_range::PatternRangePath,
        },
    },
    traversal::{
        cache::{
            key::{
                root::RootKey,
                DirectedKey,
            },
            TraversalCache,
        },
        fold::state::{
            FinalState,
            FoldState,
        },
        result::FinishedState,
        state::end::{
            EndKind,
            EndState,
        },
    },
};
use init::{
    CursorInit,
    InitStates,
};
use std::{
    borrow::Borrow,
    fmt::Debug,
};

pub mod init;
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
        trav: &K::Trav,
    ) -> FoldResult;
}

#[macro_export]
macro_rules! impl_foldable {
    ($t:ty, $f:ident) => {
        impl Foldable for $t {
            fn fold<'a, K: TraversalKind>(
                self,
                trav: &'a K::Trav,
            ) -> FoldResult {
                FoldContext::<'a, K>::$f(trav, self)
            }
        }
    };
}
//impl_foldable!(QueryState, fold_query);
impl_foldable!(PatternRangePath, fold_path);
impl_foldable!(PatternPrefixPath, fold_path);
impl_foldable!(Pattern, fold_pattern);

/// context for running fold traversal
#[derive(Debug)]
pub struct FoldContext<'a, K: TraversalKind> {
    pub trav: &'a K::Trav,
    pub start_index: Child,

    pub max_width: usize,
    pub end_state: Option<EndState>,
    pub states: StatesContext<K>,
}
impl<K: TraversalKind> Iterator for FoldContext<'_, K> {
    type Item = (usize, TraversalState);

    fn next(&mut self) -> Option<Self::Item> {
        self.states.next()
    }
}
impl<'a, K: TraversalKind> FoldContext<'a, K> {
    pub fn fold_pattern<P: IntoPattern>(
        trav: &'a K::Trav,
        pattern: P,
    ) -> FoldResult {
        let pattern = pattern.into_pattern();

        // build cursor path
        let path = PatternRangePath::from(pattern.borrow());

        Self::fold_path(trav, path)
    }
    pub fn fold_path(
        trav: &'a K::Trav,
        query: impl FoldablePath,
    ) -> Result<FinishedState, ErrorState> {
        Self::fold_cursor(trav, query.to_range_path().to_cursor())
    }
    pub fn fold_cursor(
        trav: &'a K::Trav,
        cursor: RangeCursor,
    ) -> Result<FinishedState, ErrorState> {
        let init = CursorInit::<K> {
            trav,
            cursor: &cursor,
        };
        let start_index = init.cursor.path.start_index(trav);

        let mut ctx = Self {
            states: init.init_context(),
            trav,
            end_state: None,
            max_width: start_index.width(),
            start_index,
        };
        ctx.fold_states()?;
        ctx.finish_fold(cursor.path)
    }
    fn fold_states(&mut self) -> Result<(), ErrorState> {
        while let Some((depth, tstate)) = self.next() {
            let mut ctx = TraversalContext::<K> {
                trav: self.trav,
                states: &mut self.states,
            };
            if let Some(next_states) = tstate.next_states(&mut ctx) {
                if (ApplyStatesCtx {
                    tctx: &mut ctx,
                    max_width: &mut self.max_width,
                    end_state: &mut self.end_state,
                    depth,
                })
                .apply_transition(next_states)
                .is_break()
                {
                    break;
                }
            }
        }
        Ok(())
    }
    fn finish_fold(
        self,
        path: PatternRangePath,
    ) -> Result<FinishedState, ErrorState> {
        if let Some(state) = self.end_state {
            Ok(FoldFinished {
                end_state: state,
                cache: self.states.cache,
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
