use crate::{
    path::structs::query_range_path::FoldablePath,
    traversal::{
        cache::key::directed::up::UpKey,
        container::extend::ExtendStates,
        fold::states::PrunedStates,
        state::{
            bottom_up::start::StartState,
            cursor::PatternRangeCursor,
        },
        TraversalKind,
    },
};
pub trait MakeStartState {
    fn start_state(&self) -> StartState;
}
impl<K: TraversalKind> MakeStartState for CursorInit<'_, K> {
    fn start_state(&self) -> StartState {
        let start_index = self.cursor.path.start_index(self.trav);

        StartState {
            index: start_index,
            key: UpKey::new(start_index, 0.into()),
            cursor: self.cursor.clone(),
        }
    }
}
pub struct CursorInit<'a, K: TraversalKind> {
    pub trav: &'a K::Trav,
    pub cursor: &'a PatternRangeCursor,
}
impl<K: TraversalKind> From<CursorInit<'_, K>> for PrunedStates<K> {
    fn from(init: CursorInit<'_, K>) -> Self {
        let first = init
            .start_state()
            .next_states::<K>(init.trav)
            .into_iter()
            .map(|n| (1, n));

        let mut states = Self::default();
        states.extend(first);
        states
    }
}
