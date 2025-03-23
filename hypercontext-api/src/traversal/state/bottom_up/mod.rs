use std::collections::VecDeque;

use parent::ParentState;
use start::StartState;

use crate::traversal::TraversalKind;

use super::{
    top_down::end::EndState,
    StateNext,
};

pub mod parent;
pub(crate) mod start;

#[derive(Clone, Debug)]
pub enum BUNext {
    Parents(StateNext<Vec<ParentState>>),
    End(StateNext<EndState>),
}

pub struct BUIter<K: TraversalKind> {
    pub trav: K::Trav,
    pub states: VecDeque<StateNext<ParentState>>,
}

//impl<K: TraversalKind> Iterator for BUIter<K> {
//    type Item = BUNext;
//
//    fn next(&mut self) -> Option<Self::Item> {
//        self.states
//            .pop_front()
//            .map(|p| p.inner.parent_next_states(&self.trav, p.prev))
//    }
//}

impl<K: TraversalKind> TryFrom<(K::Trav, StartState)> for BUIter<K> {
    type Error = StateNext<EndState>;
    fn try_from((trav, start): (K::Trav, StartState)) -> Result<Self, Self::Error> {
        let next = start.next_states::<K>(&trav);
        match next {
            BUNext::End(end) => Err(end),
            BUNext::Parents(next) => Ok(Self {
                trav,
                // TODO: Create caches from next.prev
                states: FromIterator::from_iter(next.inner.into_iter().map(|inner| StateNext {
                    prev: next.prev,
                    inner,
                })),
            }),
        }
    }
}
