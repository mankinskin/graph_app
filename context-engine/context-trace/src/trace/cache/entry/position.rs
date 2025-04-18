use std::num::NonZeroUsize;

use super::new::NewEntry;
use crate::{
    HashMap,
    HashSet,
    graph::vertex::{
        child::Child,
        location::SubLocation,
    },
    trace::{
        StateDirection,
        cache::{
            TraceCache,
            key::directed::DirectedKey,
        },
        traversable::Traversable,
    },
};

pub type Offset = NonZeroUsize;

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
}

impl PositionCache {
    pub fn start(index: Child) -> Self {
        Self {
            index,
            edges: Default::default(),
        }
    }
    pub fn new<Trav: Traversable>(
        cache: &mut TraceCache,
        trav: &Trav,
        key: DirectedKey,
        state: NewEntry,
        add_edges: bool,
    ) -> Self {
        let mut edges = Edges::default();
        match (add_edges, state.state_direction(), state.entry_location()) {
            (true, StateDirection::BottomUp, Some(entry)) => {
                edges.bottom.insert(
                    state.prev_key().prev_target,
                    entry.to_sub_location(),
                );
            },
            (true, StateDirection::TopDown, Some(entry)) => {
                let prev = cache.force_mut(trav, &state.prev_key().prev_target);
                prev.edges.bottom.insert(key, entry.to_sub_location());
            },
            _ => {},
        }
        Self {
            index: key.index,
            edges,
        }
    }
    pub fn num_parents(&self) -> usize {
        self.edges.top.len()
    }
    pub fn num_bu_edges(&self) -> usize {
        self.edges.bottom.len()
    }
}
