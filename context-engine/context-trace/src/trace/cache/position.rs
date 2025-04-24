use std::num::NonZeroUsize;

use super::new::EditKind;
use crate::{
    HashMap,
    HashSet,
    graph::vertex::{
        child::Child,
        location::{
            SubLocation,
            child::ChildLocation,
        },
    },
    trace::cache::{
        TraceCache,
        key::directed::DirectedKey,
    },
};

pub type Offset = NonZeroUsize;

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
pub enum AddChildLocation {
    Target(ChildLocation),
    Prev(ChildLocation),
}
impl PositionCache {
    pub fn start(index: Child) -> Self {
        Self {
            index,
            edges: Default::default(),
        }
    }
    pub fn new(
        cache: &mut TraceCache,
        key: DirectedKey,
        state: EditKind,
        add_edges: bool,
    ) -> Self {
        // create all bottom edges (created upwards or downwards)
        let mut edges = Edges::default();
        match (add_edges, state) {
            (false, _) => {},
            (_, EditKind::Parent(edit)) => {
                // created by upwards traversal
                edges.bottom.insert(
                    edit.target.into(),
                    edit.location.to_sub_location(),
                );
            },
            (_, EditKind::Child(edit)) => {
                // created by downwards traversal
                let prev = cache.force_mut(&(edit.prev.into()));
                prev.edges.bottom.insert(
                    edit.target.into(),
                    edit.location.to_sub_location(),
                );
            },
            //(_, EditKind::Root(edit)) => {
            //    //let prev = cache.force_mut(&state.prev);
            //    //prev.edges
            //    //    .bottom
            //    //    .insert(key.clone(), edit.entry.to_sub_location());
            //},
        }
        //match (add_edges, state.state_direction(), state.entry_location()) {
        //    (true, StateDirection::BottomUp, Some(entry)) => {
        //        edges.bottom.insert(state.prev, entry.to_sub_location());
        //    },
        //    (true, StateDirection::TopDown, Some(entry)) => {
        //        let prev = cache.force_mut(&state.prev);
        //        prev.edges
        //            .bottom
        //            .insert(key.clone(), entry.to_sub_location());
        //    },
        //    _ => {},
        //}
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
