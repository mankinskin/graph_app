use super::*;

pub mod entry;
pub use entry::*;
pub mod state;
pub use state::*;
pub mod key;
pub use key::*;

type HashMap<K, V> = DeterministicHashMap<K, V>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TraversalCache {
    pub(crate) query_root: Pattern,
    pub(crate) entries: HashMap<VertexIndex, PositionCache>,
}
impl TraversalCache {
    pub fn new(start: &StartState, query: Pattern) -> (CacheKey, Self) {
        let mut entries = HashMap::default();
        entries.insert(start.index.index(), PositionCache::start(start.index));
        (CacheKey::new(start.index, 0), Self {
            query_root: query,
            entries,
        })
    }
    pub fn get_entry(&self, key: &CacheKey) -> Option<&PositionCache> {
        self.entries.get(&key.index.index())
    }
    pub fn get_entry_mut(&mut self, key: &CacheKey) -> Option<&mut PositionCache> {
        self.entries.get_mut(&key.index.index())
    }
    /// adds node to cache and returns the state of the insertion
    pub fn add_state<
        Trav: Traversable,
    >(&mut self, trav: &Trav, state: &TraversalState) -> Result<CacheKey, CacheKey> {
        let key = state.target_key(trav);
        if let Some(_) = self.entries.get_mut(&key.index.index()) {
            Err(key)
        } else {
            self.new_vertex(
                trav,
                key, 
                state,
            );
            Ok(key)
        }
    }
    fn new_vertex<
        Trav: Traversable,
    >(
        &mut self,
        trav: &Trav,
        key: CacheKey,
        state: &TraversalState,
    ) {
        let ve = PositionCache::new(
            trav,
            state
        );
        //let mut ve = VertexCache {
        //    positions: Default::default()
        //};
        //ve.new_position(
        //    key,
        //    state,
        //);
        self.entries.insert(key.index.index(), ve);
    }
    pub fn continue_waiting(
        &mut self,
        key: &CacheKey,
    ) -> Vec<(usize, TraversalState)> {
        self.get_entry_mut(key)
            .unwrap()
            .waiting
            .drain(..)
            .map(|(d, s)| (d, s.into()))
            .collect()
    }
}