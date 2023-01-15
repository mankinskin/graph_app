use super::*;

pub mod entry;
pub use entry::*;
pub mod state;
pub use state::*;
pub mod key;
pub use key::*;

type HashMap<K, V> = DeterministicHashMap<K, V>;

#[derive(Clone, Debug)]
pub struct TraversalCache<R: ResultKind, Q: BaseQuery> {
    entries: HashMap<usize, VertexCache<R, Q>>,
}
impl<R: ResultKind, Q: BaseQuery> TraversalCache<R, Q> {
    pub fn new() -> Self {
        Self {
            entries: Default::default(),
        }
    }
    pub fn get_entry_mut(&mut self, key: &CacheKey) -> Option<&mut PositionCache<R, Q>> {
        self.entries.get_mut(&key.index)
            .and_then(|e|
                e.positions.get_mut(&key.token_pos)
            )
    }
    /// adds node to cache and returns the state of the insertion
    pub fn add_state(&mut self, state: &TraversalState<R, Q>) -> Result<CacheKey, CacheKey> {
        let key = state.leaf_key();
        if let Some(ve) = self.entries.get_mut(&key.index) {
            if let Some(_) = ve.positions.get_mut(&key.token_pos) {
                Err(key)
            } else {
                ve.new_position(
                    key,
                    state,
                );
                Ok(key)
            }
        } else {
            self.new_vertex(
                key, 
                state,
            );
            Ok(key)
        }
    }
    fn new_vertex(
        &mut self,
        key: CacheKey,
        state: &TraversalState<R, Q>,
    ) {
        let mut ve = VertexCache {
            positions: Default::default()
        };
        ve.new_position(
            key,
            state,
        );
        self.entries.insert(key.index, ve);
    }
    pub fn continue_waiting(
        &mut self,
        key: &CacheKey,
    ) -> Vec<(usize, TraversalState<R, Q>)> {
        self.get_entry_mut(key)
            .unwrap()
            .waiting
            .drain(..).collect()
    }
}