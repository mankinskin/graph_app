use std::collections::VecDeque;

use crate::traversal::{
    compare::parent::ParentCompareState,
    iterator::r#match::{
        RootSearchIterator,
        TraceNode::Parent,
    },
    state::{
        cursor::PatternPrefixCursor,
        end::EndState,
    },
    MatchContext,
    TraversalKind,
};
use context_trace::trace::{
    state::parent::ParentBatch,
    TraceContext,
};
use derive_more::{
    Deref,
    DerefMut,
};
use derive_new::new;
#[derive(Debug, new)]
pub struct EndIterator<K: TraversalKind>(
    pub TraceContext<K::Trav>,
    pub MatchContext,
);

impl<K: TraversalKind> EndIterator<K> {
    pub fn find_next(&mut self) -> Option<EndState> {
        self.find_map(|flow| Some(flow))
    }
}
#[derive(Debug, Clone, Deref, DerefMut)]
pub struct CompareParentBatch {
    #[deref]
    #[deref_mut]
    pub batch: ParentBatch,
    pub cursor: PatternPrefixCursor,
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
impl<K: TraversalKind> Iterator for EndIterator<K> {
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
