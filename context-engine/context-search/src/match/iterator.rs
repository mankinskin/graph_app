use std::collections::VecDeque;

use crate::{
    compare::parent::ParentCompareState,
    r#match::{
        MatchCtx,
        RootSearchIterator,
        TraceNode::Parent,
    },
    traversal::{
        state::{
            cursor::PatternCursor,
            end::EndState,
        },
        TraversalKind,
    },
};
use context_trace::trace::{
    state::parent::ParentBatch,
    TraceCtx,
};
use derive_more::{
    Deref,
    DerefMut,
};
use derive_new::new;

#[derive(Debug, Clone, Deref, DerefMut)]
pub struct CompareParentBatch {
    #[deref]
    #[deref_mut]
    pub batch: ParentBatch,
    pub cursor: PatternCursor,
}
impl CompareParentBatch {
    pub fn into_compare_batch(self) -> VecDeque<ParentCompareState> {
        self.batch
            .parents
            .into_iter()
            .map(|parent_state| ParentCompareState {
                parent_state,
                cursor: self.cursor.clone(),
            })
            .collect()
    }
}

#[derive(Debug, new)]
pub struct MatchIterator<K: TraversalKind>(pub TraceCtx<K::Trav>, pub MatchCtx);

impl<K: TraversalKind> MatchIterator<K> {
    pub fn find_next(&mut self) -> Option<EndState> {
        self.find_map(|flow| Some(flow))
    }
}

impl<K: TraversalKind> Iterator for MatchIterator<K> {
    type Item = EndState;

    fn next(&mut self) -> Option<Self::Item> {
        match RootSearchIterator::<K>::new(&self.0.trav, &mut self.1)
            .find_root_cursor()
        {
            Some(root_cursor) => Some({
                match root_cursor.find_end() {
                    Ok(end) => end,
                    Err(root_cursor) =>
                        match root_cursor.next_parents::<K>(&self.0.trav) {
                            Err(end) => end,
                            Ok((parent, batch)) => {
                                assert!(!batch.is_empty());
                                // next batch
                                self.1.nodes.extend(
                                    batch
                                        .into_compare_batch()
                                        .into_iter()
                                        .map(Parent),
                                );
                                EndState::mismatch(&self.0.trav, parent)
                            },
                        },
                }
                //debug!("End {:#?}", end);
            }),
            None => None,
        }
    }
}
