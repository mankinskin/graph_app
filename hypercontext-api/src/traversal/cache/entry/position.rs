use crate::{
    HashMap,
    HashSet,
    traversal::{
        cache::{
            entry::{
                NewEntry,
                Offset,
                StateDepth,
            },
            key::DirectedKey,
            state::{
                StateDirection,
                WaitingState,
            },
            TraversalCache,
        },
        traversable::Traversable,
    },
};
use crate::graph::vertex::{
    child::Child,
    location::SubLocation,
};

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
    pub fn new<Trav: Traversable>(
        cache: &mut TraversalCache,
        trav: &Trav,
        key: DirectedKey,
        state: NewEntry,
        add_edges: bool,
    ) -> Self {
        let mut edges = Edges::default();
        match (add_edges, state.state_direction(), state.entry_location()) {
            (true, StateDirection::BottomUp, Some(entry)) => {
                edges
                    .bottom
                    .insert(state.prev_key().prev_target, entry.to_sub_location());
            }
            (true, StateDirection::TopDown, Some(entry)) => {
                let prev = cache.force_mut(trav, &state.prev_key().prev_target);
                prev.edges.bottom.insert(key, entry.to_sub_location());
            }
            _ => {}
        }
        Self {
            index: key.index,
            edges,
            waiting: Default::default(),
        }
    }
    pub fn add_waiting(
        &mut self,
        depth: StateDepth,
        state: WaitingState,
    ) {
        self.waiting.push((depth, state));
    }
    pub fn num_parents(&self) -> usize {
        self.edges.top.len()
    }
    pub fn num_bu_edges(&self) -> usize {
        self.edges.bottom.len()
    }
}
