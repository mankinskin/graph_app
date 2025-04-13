use crate::{
    graph::{
        getters::ErrorReason,
        vertex::child::Child,
    },
    path::mutators::{
        adapters::IntoPrimer,
        move_path::Advance,
    },
    traversal::{
        fold::foldable::ErrorState,
        iterator::policy::DirectedTraversalPolicy,
        result::FinishedKind,
        state::cursor::PatternRangeCursor,
        ParentBatch,
        TraversalKind,
    },
};

#[derive(Debug, PartialEq, Eq)]
pub struct StartCtx<K: TraversalKind> {
    pub index: Child,
    pub cursor: PatternRangeCursor,
    pub trav: K::Trav,
}

impl<K: TraversalKind> StartCtx<K> {
    pub fn get_parent_batch(&self) -> Result<ParentBatch, ErrorState> {
        let mut cursor = self.cursor.clone();
        if cursor.advance(&self.trav).is_continue() {
            //prev: self.key.to_prev(delta),
            Ok(K::Policy::gen_parent_batch(
                &self.trav,
                self.index,
                |trav, p| (self.index, cursor.clone()).into_primer(trav, p),
            ))
        } else {
            Err(ErrorState {
                reason: ErrorReason::SingleIndex(self.index),
                found: Some(FinishedKind::Complete(self.index)),
            })
        }
    }
}

//impl RootKey for StartState {
//    fn root_key(&self) -> UpKey {
//        UpKey::new(self.index, TokenPosition(self.index.width()).into())
//    }
//}
