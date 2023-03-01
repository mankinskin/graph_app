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
    pub(crate)fn new_position(
        &mut self,
        key: CacheKey,
        state: &TraversalState,
    ) {
        let ve = PositionCache::new(
            key,
            state
        );
        self.positions.insert(
            key.pos,
            ve,
        );
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
            // todo: update num_parents when creating forward edges
            edges: Default::default(),
            waiting: Default::default(),
        }
    }
    pub fn new(
        key: CacheKey,
        state: &TraversalState,
    ) -> Self {
        let mut edges = Edges::default();
        if let (prev, Some(entry)) = (state.prev_key(), state.entry_location()) {
            match state.node_direction() {
                NodeDirection::TopDown => {
                    edges.top.insert(prev, entry.to_sub_location());
                },
                NodeDirection::BottomUp => {
                    edges.bottom.insert(prev);
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