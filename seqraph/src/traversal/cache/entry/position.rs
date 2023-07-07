use crate::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BottomUp;
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TopDown;

pub trait TraversalDirection {
    type Opposite: TraversalDirection;
}
impl TraversalDirection for BottomUp {
    type Opposite = TopDown;
}
impl TraversalDirection for TopDown {
    type Opposite = BottomUp;
}
/// optional offset inside of pattern sub location
/// 
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SubSplitLocation {
    pub location: SubLocation,
    pub inner_offset: Option<Offset>,
}
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct Edges {
    pub top: HashSet<DirectedKey>,
    pub bottom: HashMap<DirectedKey, SubLocation>,
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
        prev: Option<&mut PositionCache>,
        key: DirectedKey,
        state: NewEntry,
    ) -> Self {
        let mut edges = Edges::default();
        match (state.state_direction(), prev, state.entry_location()) {
            (StateDirection::BottomUp, Some(_prev), Some(entry)) => {
                edges.bottom.insert(state.prev_key(), entry.to_sub_location());
            },
            (StateDirection::TopDown, Some(prev), Some(entry)) => {
                prev.edges.bottom.insert(key, entry.to_sub_location());
            },
            _ => {},
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