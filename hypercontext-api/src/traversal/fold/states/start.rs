use crate::{
    path::structs::query_range_path::FoldablePath,
    traversal::{
        cache::key::directed::up::UpKey,
        state::bottom_up::start::StartState,
        TraversalKind,
    },
};

use super::init::CursorInit;
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
