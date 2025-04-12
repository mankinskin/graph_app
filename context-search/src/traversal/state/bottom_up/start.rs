use crate::{
    graph::{
        getters::ErrorReason,
        vertex::{
            child::Child,
            wide::Wide,
        },
    },
    impl_cursor_pos,
    path::mutators::{
        adapters::IntoPrimer,
        move_path::{
            key::TokenPosition,
            Advance,
        },
    },
    traversal::{
        cache::key::{
            directed::up::UpKey,
            props::RootKey,
        },
        fold::foldable::ErrorState,
        iterator::policy::DirectedTraversalPolicy,
        result::FinishedKind,
        state::cursor::PatternRangeCursor,
        ParentBatch,
        TraversalKind,
    },
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StartContext<K: TraversalKind> {
    pub trav: K::Trav,
    pub state: StartState,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StartState {
    pub index: Child,
    pub key: UpKey,
    pub cursor: PatternRangeCursor,
}

impl_cursor_pos! {
    CursorPosition for StartState, self => self.cursor.relative_pos
}
impl StartState {
    pub fn get_parent_batch<'a, K: TraversalKind>(
        &self,
        trav: &K::Trav,
    ) -> Result<ParentBatch, ErrorState>
    where
        Self: 'a,
    {
        //let delta = self.index.width();
        let mut cursor = self.cursor.clone();
        if cursor.advance(trav).is_continue() {
            //prev: self.key.to_prev(delta),
            Ok(K::Policy::gen_parent_batch(trav, self.index, |trav, p| {
                (self.index, cursor.clone()).into_primer(trav, p)
            }))
        } else {
            Err(ErrorState {
                reason: ErrorReason::SingleIndex(self.index),
                found: Some(FinishedKind::Complete(self.index)),
            })
        }
    }
}

impl RootKey for StartState {
    fn root_key(&self) -> UpKey {
        UpKey::new(self.index, TokenPosition(self.index.width()).into())
    }
}
