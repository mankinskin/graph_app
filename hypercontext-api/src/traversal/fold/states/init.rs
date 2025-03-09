use crate::traversal::{
    container::extend::ExtendStates,
    fold::states::{
        start::MakeStartState,
        StatesContext,
    },
    state::cursor::PatternRangeCursor,
    TraversalKind,
};
pub struct CursorInit<'a, K: TraversalKind> {
    pub trav: &'a K::Trav,
    pub cursor: &'a PatternRangeCursor,
}
impl<K: TraversalKind> From<CursorInit<'_, K>> for StatesContext<K> {
    fn from(init: CursorInit<'_, K>) -> Self {
        let fan = init
            .start_state()
            .next_states::<K>(init.trav)
            .into_iter()
            .map(|n| (1, n));

        let mut ctx = Self::default();
        ctx.extend(fan);
        ctx
    }
}
