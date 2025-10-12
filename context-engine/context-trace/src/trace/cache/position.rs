use std::num::NonZeroUsize;

use crate::trace::new::{EditKind};
use crate::{
    HashMap,
    HashSet,
    graph::vertex::location::{
        SubLocation,
        child::ChildLocation,
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
pub struct PositionCache {
    pub top: HashSet<DirectedKey>,
    pub bottom: HashMap<DirectedKey, SubLocation>,
}
pub enum AddChildLocation {
    Target(ChildLocation),
    Prev(ChildLocation),
}
impl PositionCache {
    pub fn new(
        cache: &mut TraceCache,
        state: EditKind,
        add_edges: bool,
    ) -> Self {
        // create all bottom edges (created upwards or downwards)
        let mut bottom = HashMap::default();
        match (add_edges, state) {
            (false, _) => {},
            (_, EditKind::Parent(edit)) => {
                // created by upwards traversal
                bottom
                    .insert(edit.prev.into(), edit.location.to_sub_location());
            },
            (_, EditKind::Child(edit)) => {
                // created by downwards traversal
                let prev = cache.force_mut(&(edit.prev.into()));
                prev.bottom.insert(
                    edit.target.into(),
                    edit.location.to_sub_location(),
                );
            },
        }
        Self {
            bottom,
            top: HashSet::default(),
        }
    }
    pub fn num_parents(&self) -> usize {
        self.top.len()
    }
    pub fn num_bu_edges(&self) -> usize {
        self.bottom.len()
    }
}
