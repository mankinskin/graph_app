use crate::{
    traversal::cache::{
        entry::VertexCache,
        key::{
            DirectedKey,
            TargetKey,
        },
        state::end::EndState,
        TraversalCache,
    },
    vertex::{
        child::Child,
        indexed::Indexed,
        wide::Wide,
    },
};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum RootMode {
    Prefix,
    Postfix,
    Infix,
}
impl Default for RootMode {
    fn default() -> Self {
        Self::Infix
    }
}
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum FoldResult {
    Complete(Child),
    Incomplete(FoldState),
}
impl FoldResult {
    pub fn unwrap_complete(self) -> Child {
        match self {
            Self::Complete(c) => c,
            _ => panic!("Unable to unwrap complete FoldResult"),
        }
    }
    pub fn unwrap_incomplete(self) -> FoldState {
        match self {
            Self::Incomplete(s) => s,
            _ => panic!("Unable to unwrap incomplete FoldResult"),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct FoldState {
    pub cache: TraversalCache,
    pub end_state: EndState,
    pub(crate) start: Child,
    pub(crate) root: Child,
}
impl FoldState {
    pub fn root_entry(&self) -> &VertexCache {
        self.cache.entries.get(&self.root().vertex_index()).unwrap()
    }
    //pub fn root_mode(&self) -> RootMode {
    //    let e = self.root_entry();
    //    if e.bottom_up.is_empty() {
    //        assert!(!e.top_down.is_empty());
    //        RootMode::Prefix
    //    } else if e.top_down.is_empty() {
    //        RootMode::Postfix
    //    } else {
    //        RootMode::Infix
    //    }
    //}
    pub fn start_key(&self) -> DirectedKey {
        DirectedKey::new(self.start, self.start.width())
    }
    pub fn root(&self) -> Child {
        self.root
    }
    pub fn into_fold_result(self) -> FoldResult {
        FoldResult::Incomplete(self)
    }
    pub fn leaf(&self) -> DirectedKey {
        self.end_state.target_key()
        //.iter()
        //.filter(|s| s.root_key().index == *root)
        //.map(|s| s.target_key())
        //.collect()
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
