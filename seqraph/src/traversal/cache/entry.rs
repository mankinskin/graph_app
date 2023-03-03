use super::*;
type StateDepth = usize;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VertexCache {
    pub(crate) positions: HashMap<TokenLocation, PositionCache>
}
impl VertexCache {
    pub fn start(index: Child) -> Self {
        let mut positions = HashMap::default();
        positions.insert(
            index.width().into(),
            PositionCache::start(index)
        );
        Self {
            positions,
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct Edges {
    pub top: HashMap<CacheKey, SubLocation>,
    pub bottom: HashSet<CacheKey>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PositionCache {
    pub edges: Edges,
    pub index: Child,
    pub waiting: Vec<(StateDepth, WaitingState)>,
}
impl PositionCache {
    pub fn start(index: Child) -> Self {
        Self {
            index,
            edges: Default::default(),
            waiting: Default::default(),
        }
    }
    pub fn new(
        prev: &mut PositionCache,
        key: CacheKey,
        state: &TraversalState,
    ) -> Self {
        let mut edges = Edges::default();
        if let Some(entry) = state.entry_location() {
            match state.node_direction() {
                NodeDirection::TopDown => {
                    edges.top.insert(state.prev_key(), entry.to_sub_location());
                    prev.edges.bottom.insert(key);
                },
                NodeDirection::BottomUp => {
                    edges.bottom.insert(state.prev_key());
                    prev.edges.top.insert(key, entry.to_sub_location());
                },
            }
        }
        Self {
            index: key.index,
            edges,
            waiting: Default::default(),
        }
    }
    pub fn add_waiting(&mut self, depth: StateDepth, state: WaitingState) {
        self.waiting.push((depth, state));
    }
    pub fn num_parents(&self) -> usize {
        self.edges.top.len()
    }
    pub fn num_bu_edges(&self) -> usize {
        self.edges.bottom.len()
    }
}