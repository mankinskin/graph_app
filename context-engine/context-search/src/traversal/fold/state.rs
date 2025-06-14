use std::cmp::Ordering;

use crate::traversal::EndState;
use context_trace::{
    graph::vertex::{
        child::Child,
        has_vertex_index::HasVertexIndex,
        wide::Wide,
    },
    trace::cache::{
        key::{
            directed::DirectedKey,
            props::RootKey,
        },
        vertex::VertexCache,
        TraceCache,
    },
};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct FinalState<'a> {
    pub num_parents: usize,
    pub state: &'a EndState,
}

impl PartialOrd for FinalState<'_> {
    fn partial_cmp(
        &self,
        other: &Self,
    ) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for FinalState<'_> {
    fn cmp(
        &self,
        other: &Self,
    ) -> Ordering {
        self.num_parents.cmp(&other.num_parents).then_with(|| {
            other
                .state
                .is_complete()
                .cmp(&self.state.is_complete())
                .then_with(|| {
                    other
                        .state
                        .root_key()
                        .width()
                        .cmp(&self.state.root_key().width())
                })
        })
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct FoldState {
    pub cache: TraceCache,
    pub end_state: EndState,
    pub start: Child,
    pub root: Child,
}

impl FoldState {
    pub fn root_entry(&self) -> &VertexCache {
        self.cache.entries.get(&self.root().vertex_index()).unwrap()
    }
    pub fn start_key(&self) -> DirectedKey {
        DirectedKey::new(self.start, self.start.width())
    }
    pub fn root(&self) -> Child {
        self.root
    }
}

// get bottom up edge iterators
//  - use back edges for late path directly
//  - trace back edges for early path to gather bottom up edges
//    - build new cache for this or store forward edges directly in search
// edge: child location, position

// tabularize all splits bottom up
// table: location, position -> split
// breadth first bottom up traversal , merging splits
// - start walking edges up from leaf nodes
// - each edge has location in parent and position
//    - each edge defines a split in parent at location, possibly merged with nested splits from below path
//    - each node has a bottom edge n-tuple for each of its child patterns, where n is the number of splits

// - combine splits into an n+1-tuple of pieces for each split tuple and position
//    - each position needs a single n+1-tuple of pieces, built with respect to other positions
// - combine split context and all positions into pairs of halves for each position

// - continue walk up to parents, write split pieces to table for each position
//    - use table to pass finished splits upwards

// - at root, there are at least 2 splits for each child pattern and only one position
