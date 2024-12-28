use crate::{graph::vertex::child::Child, traversal::{cache::key::UpKey, container::extend::ExtendStates, state::{query::QueryState, start::StartState}}};

use super::TraversalKind;


pub trait InitStates<K: TraversalKind> {
    fn start_index(&self) -> Child;
    fn init_states(self) -> K::Container;
}
pub struct QueryStateInit<'a, K: TraversalKind> {
    pub trav: &'a K::Trav,
    pub query: &'a QueryState,
}
impl<K: TraversalKind> InitStates<K> for QueryStateInit<'_, K> {
    fn start_index(&self) -> Child {
        self.query.start_index(self.trav)
    }
    fn init_states(self) -> <K as TraversalKind>::Container {
        let start_index = self.start_index();

        let mut start = StartState {
            index: start_index,
            key: UpKey::new(
                start_index,
                0.into(),
            ),
            query: self.query.clone(),
        };
        let init = start
            .next_states::<K>(self.trav)
            .into_states()
            .into_iter()
            .map(|n| (1, n));

        let mut states = K::Container::default();
        states.extend(init);
        states
    }
}