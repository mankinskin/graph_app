use crate::{
    path::structs::query_range_path::FoldablePath,
    traversal::{
        cache::{
            key::directed::up::UpKey,
            TraversalCache,
        },
        container::extend::ExtendStates,
        state::{
            bottom_up::start::StartState,
            cursor::RangeCursor,
        },
        states::StatesContext,
        TraversalKind,
    },
};

pub trait InitStates<K: TraversalKind> {
    //fn start_index(&self) -> Child;
    fn init_context(self) -> StatesContext<K>;
}
pub struct CursorInit<'a, K: TraversalKind> {
    pub trav: &'a K::Trav,
    pub cursor: &'a RangeCursor,
}
impl<K: TraversalKind> InitStates<K> for CursorInit<'_, K> {
    //fn start_index(&self) -> Child {
    //    self.cursor.start_index(self.trav)
    //}
    fn init_context(self) -> StatesContext<K> {
        let start_index = self.cursor.path.start_index(self.trav);

        let mut start = StartState {
            index: start_index,
            key: UpKey::new(start_index, 0.into()),
            cursor: self.cursor.clone(),
        };
        let init = start
            .next_states::<K>(self.trav)
            .into_states()
            .into_iter()
            .map(|n| (1, n));

        let mut ctx = StatesContext {
            cache: TraversalCache::new(self.trav, self.cursor.path.start_index(self.trav)),
            states: K::Container::default(),
            pruning_map: Default::default(),
        };
        ctx.extend(init);
        ctx
    }
}
