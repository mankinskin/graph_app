use crate::{
    graph::vertex::{
        child::Child,
        wide::Wide,
    },
    impl_cursor_pos,
    path::mutators::{
        adapters::IntoPrimer,
        move_path::{
            key::TokenPosition,
            CanAdvance,
        },
    },
    traversal::{
        cache::key::{
            directed::up::UpKey,
            prev::ToPrev,
            props::RootKey,
        },
        iterator::policy::DirectedTraversalPolicy,
        state::{
            cursor::PatternRangeCursor,
            next_states::{
                NextStates,
                StateNext,
            },
            top_down::end::{
                EndKind,
                EndReason,
                EndState,
            },
        },
        TraversalKind,
    },
};

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
    pub fn next_states<'a, K: TraversalKind>(
        &mut self,
        trav: &K::Trav,
    ) -> NextStates
    where
        Self: 'a,
    {
        let delta = self.index.width();
        if self.cursor.can_advance(trav) {
            NextStates::Parents(StateNext {
                prev: self.key.to_prev(delta),
                //new: vec![],
                inner: K::Policy::gen_parent_states(trav, self.index, |trav, p| {
                    (self.index, self.cursor.clone()).into_primer(trav, p)
                }),
            })
        } else {
            NextStates::End(StateNext {
                prev: self.key.to_prev(delta),
                //new: vec![],
                inner: EndState {
                    reason: EndReason::QueryEnd,
                    root_pos: self.index.width().into(),
                    kind: EndKind::Complete(self.index),
                    cursor: self.cursor.clone(),
                },
            })
        }
    }
}

impl RootKey for StartState {
    fn root_key(&self) -> UpKey {
        UpKey::new(self.index, TokenPosition(self.index.width()).into())
    }
}
