use super::*;

pub mod entry;
pub use entry::*;
pub mod state;
pub use state::*;
pub mod key;
pub use key::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TraversalCache {
    pub(crate) query_root: Pattern,
    pub(crate) entries: HashMap<VertexIndex, VertexCache>,
}
impl TraversalCache {
    pub fn new(start: &StartState, query: Pattern) -> (CacheKey, Self) {
        let mut entries = HashMap::default();
        entries.insert(start.index.index(), VertexCache::start(start.index));
        (start.root_key(), Self {
            query_root: query,
            entries,
        })
    }
    pub fn get_entry(&self, key: &CacheKey) -> Option<&PositionCache> {
        self.entries.get(&key.index.index())
            .and_then(|ve|
                ve.positions.get(&key.pos)
            )
    }
    pub fn get_entry_mut(&mut self, key: &CacheKey) -> Option<&mut PositionCache> {
        self.entries.get_mut(&key.index.index())
            .and_then(|ve| {
                //println!("get_entry positions {:#?}: {:#?}", key, ve.positions);
                ve.positions.get_mut(&key.pos)
            })
    }
    /// adds node to cache and returns the state of the insertion
    pub fn add_state<
        Trav: Traversable,
    >(&mut self, trav: &Trav, state: &TraversalState) -> (CacheKey, bool) {
        let key = state.target_key(trav);
        if let Some(ve) = self.entries.get_mut(&key.index.index()) {
            if let Some(_) = ve.positions.get_mut(&key.pos) {
                (key, false)
            } else {
                ve.new_position(
                    trav,
                    key,
                    state,
                );
                (key, true)
            }
        } else {
            self.new_vertex(
                trav,
                key, 
                state,
            );
            (key, true)
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
        let mut ve = VertexCache {
            positions: Default::default()
        };
        ve.new_position(
            trav,
            key,
            state,
        );
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