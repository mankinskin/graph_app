use super::*;
type StateDepth = usize;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CacheEdge {
    TopDown(SubLocation),
    TopDownQuery(SubLocation),
    BottomUp(SubLocation),
}
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
    pub(crate)fn new_position<
        Trav: Traversable,
    >(
        &mut self,
        trav: &Trav,
        key: CacheKey,
        state: &TraversalState,
    ) {
        let ve = PositionCache::new(
            trav,
            key.index,
            state
        );
        self.positions.insert(
            key.pos,
            ve,
        );
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PositionCache {
    pub back_edges: HashMap<CacheKey, CacheEdge>,
    pub num_parents: usize,
    // for debugging
    pub index: Child,
    pub waiting: Vec<(StateDepth, WaitingState)>,
}
impl PositionCache {
    pub fn start(index: Child) -> Self {
        Self {
            index,
            // todo: update num_parents when creating forward edges
            num_parents: 0,
            back_edges: Default::default(),
            waiting: Default::default(),
        }
    }
    pub fn new<
        Trav: Traversable,
    >(
        _trav: &Trav,
        index: Child,
        state: &TraversalState,
    ) -> Self {
        let mut edges = HashMap::default();
        let num_parents = if let (prev, Some(entry)) = (state.prev_key(), state.entry_location()) {
            match state.node_direction() {
                NodeDirection::TopDown => {
                    edges.insert(prev, CacheEdge::TopDown(entry.into_sub_location()));
                    1
                },
                NodeDirection::BottomUp => {
                    edges.insert(prev, CacheEdge::BottomUp(entry.into_sub_location()));
                    0
                },
            }
        } else {
            0
        };
        Self {
            index,
            back_edges: edges,
            num_parents,
            waiting: Default::default(),
        }
    }
    pub fn add_waiting(&mut self, depth: StateDepth, state: WaitingState) {
        self.waiting.push((depth, state));
    }
    //pub fn add_back_edge(&mut self) {
    //    unimplemented!();
    //    //self.back_edges.insert();
    //}
    pub fn num_parents(&self) -> usize {
        self.num_parents
    }
    pub fn num_bu_edges(&self) -> usize {
        self.back_edges.values().filter(|v| matches!(v, CacheEdge::BottomUp(_))).count()
    }
}