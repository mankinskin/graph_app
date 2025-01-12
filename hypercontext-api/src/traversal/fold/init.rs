use crate::{graph::vertex::child::Child, traversal::{cache::{key::UpKey, TraversalCache}, container::extend::ExtendStates, state::{query::QueryState, start::StartState}, states::StatesContext, TraversalKind}};


pub trait InitStates<K: TraversalKind> {
    fn start_index(&self) -> Child;
    fn init_context(self) -> StatesContext<K>;
}
pub struct QueryStateInit<'a, K: TraversalKind> {
    pub trav: &'a K::Trav,
    pub query: &'a QueryState,
}
impl<K: TraversalKind> InitStates<K> for QueryStateInit<'_, K> {
    fn start_index(&self) -> Child {
        self.query.start_index(self.trav)
    }
    fn init_context(self) -> StatesContext<K> {
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

        let mut ctx = StatesContext {
            cache: TraversalCache::new(self.trav, self.start_index()),
            states: K::Container::default(),
            pruning_map: Default::default(),
        };
        ctx.extend(init);
        ctx
    }
}