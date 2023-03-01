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
    pub fn get(&self, key: &CacheKey) -> Option<&PositionCache> {
        self.entries.get(&key.index.index())
            .and_then(|ve|
                ve.positions.get(&key.pos)
            )
    }
    pub fn get_mut(&mut self, key: &CacheKey) -> Option<&mut PositionCache> {
        self.entries.get_mut(&key.index.index())
            .and_then(|ve| {
                //println!("get_entry positions {:#?}: {:#?}", key, ve.positions);
                ve.positions.get_mut(&key.pos)
            })
    }
    pub fn expect(&self, key: &CacheKey) -> &PositionCache {
        self.get(key).unwrap()
    }
    pub fn expect_mut(&mut self, key: &CacheKey) -> &mut PositionCache {
        self.get_mut(key).unwrap()
    }
    /// adds node to cache and returns the state of the insertion
    pub fn add_state(&mut self, state: &TraversalState) -> (CacheKey, bool) {
        let key = state.target_key();
        if let Some(ve) = self.entries.get_mut(&key.index.index()) {
            if let Some(_) = ve.positions.get_mut(&key.pos) {
                (key, false)
            } else {
                ve.new_position(
                    key,
                    state,
                );
                let prev = state.prev_key();
                match (state.node_direction(), state.entry_location()) {
                    (NodeDirection::TopDown, Some(_)) => {
                        self.expect_mut(&prev).edges.bottom.insert(key);
                    },
                    (NodeDirection::BottomUp, Some(entry)) => {
                        self.expect_mut(&prev).edges.top.insert(key, entry.to_sub_location());
                    },
                    _ => {}
                }
                (key, true)
            }
        } else {
            self.new_vertex(
                key, 
                state,
            );
            (key, true)
        }
    }
    fn new_vertex(
        &mut self,
        key: CacheKey,
        state: &TraversalState,
    ) {
        let mut ve = VertexCache {
            positions: Default::default()
        };
        ve.new_position(
            key,
            state,
        );
        self.entries.insert(key.index.index(), ve);
    }
    //pub fn trace_down_from(
    //    &mut self,
    //    key: &CacheKey,
    //) {
    //    let mut queue = VecDeque::new();
    //    let mut node = self.expect_entry_mut(key);
    //    queue.extend(node.edges.bottom.iter());
    //    let mut prev = key;
    //    while let Some(node_key) = queue.pop_front() {
    //        let mut node = self.expect_entry_mut(node_key);
    //        node.edges.top.insert(*prev);
    //        queue.extend(node.edges.bottom.iter());
    //    }
    //}
    pub fn continue_waiting(
        &mut self,
        key: &CacheKey,
    ) -> Vec<(usize, TraversalState)> {
        self.get_mut(key)
            .unwrap()
            .waiting
            .drain(..)
            .map(|(d, s)| (d, s.into()))
            .collect()
    }
    //pub fn trace_subgraph(&mut self, end_states: &Vec<EndState>) {
    //    for state in end_states {
    //        self.trace_down_from(&state.root_key())    
    //    }
    //}
}