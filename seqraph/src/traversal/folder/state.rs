use crate::*;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct FoldState {
    pub cache: TraversalCache,
    pub end_states: Vec<EndState>,
    pub(crate) start: Child,
    pub(crate) root: Child,
    pub(crate) end_pos: TokenLocation,
}
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
impl FoldState {
    pub fn root_entry(&self) -> &VertexCache {
        self.cache.entries.get(&self.root().index()).unwrap()
    }
    pub fn root_mode(&self) -> RootMode {
        let e = self.root_entry();
        if e.bottom_up.is_empty() {
            assert!(!e.top_down.is_empty());
            RootMode::Prefix
        } else if e.top_down.is_empty() {
            RootMode::Postfix
        } else {
            RootMode::Infix
        }
    }
    pub fn start_key(&self) -> DirectedKey {
        DirectedKey::new(self.start, self.start.width())
    }
    pub fn root(&self) -> Child {
        self.root
    }
    pub fn into_fold_result(self) -> FoldResult {
        FoldResult::Incomplete(self)
    }
    pub fn leaves(&self, root: &Child) -> Vec<DirectedKey> {
        self.end_states.iter()
            .filter(|s| s.root_key().index == *root)
            .map(|s| s.target_key())
            .collect()
    }
    pub fn into_split_graph<'a, Trav: TraversableMut<GuardMut<'a> = RwLockWriteGuard<'a, Hypergraph>> + 'a>(
        &mut self,
        trav: &'a mut Trav,
    ) -> SplitCache {
        let mut cache = SplitCache::new(
            self,
            trav,
        );
        let graph = trav.graph_mut();
        cache.complete_root(
            TraceContext::new(
                &graph,
                self.root,
            ),
            self.root_mode(),
        );
        // stores past states
        let mut incomplete = BTreeSet::<Child>::default();
        // traverse top down by width
        // cache indices without range infos
        // calc range infos for larger indices when smaller index is traversed
        while let Some(state) = cache.states.pop_front() {
            // trace offset splits top down by width
            // complete past states larger than current state
            // store offsets and filter leaves
            cache.trace(&graph, self, &state);
            incomplete.insert(state.index);
            let complete = incomplete.split_off(&ChildWidth(state.index.width() + 1));
            for c in complete {
                let new = cache.complete_node(
                    TraceContext::new(
                        &graph,
                        c,
                    ),
                );
                // todo: force order
                cache.states.extend(new.into_iter());
            }
        };
        cache
    }
    pub fn complete_splits<Trav: Traversable, N: NodeType>(
        &self,
        trav: &Trav,
        index: &Child,
    ) -> N::CompleteSplitOutput {
        self.cache.entries.get(&index.index()).map(|e|
            e.complete_splits::<_, N>(
                trav,
                self.end_pos,
            )
        )
        .unwrap_or_default()
    }
    pub fn child_trace_states<Trav: Traversable>(
        &mut self,
        trav: &Trav,
        index: &Child,
        cache: &mut SplitCache
    ) -> Vec<TraceState> {
        let subs =
            self.complete_splits::<_, InnerNode>(
                trav,
                index,
            )
            .into_iter()
            .filter_map(|(parent_offset, res)|
                res.ok().map(|locs|
                    (parent_offset, locs)
                )
            );
        cache.leaves.filter_trace_states(
            trav,
            index,
            subs,
        )
    }
    //pub fn trace_entry<Trav: Traversable>(
    //    &mut self,
    //    trav: &Trav,
    //    frontier: &mut VecDeque<TraceState>,
    //    index: &Child,
    //) {
    //    let graph = trav.graph();
    //    let entry = self.cache.entries.get(&index.index).unwrap();
    //    let (_, node) = graph.expect_vertex(index);
    //}
    //pub fn trace_subgraph<Trav: Traversable>(&mut self, trav: &Trav) {
    //    let root = self.root();
    //    let mut frontier = VecDeque::new();
    //    // 1. find split position for each pattern missing a bottom edge
    //    // 2. create entry for child if missing
    //    // 3. repeat for all children until finished (needs frontier)
    //    // 4. root might have 2 children per pattern
    //    // 5. 
    //    
    //}
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum FoldResult {
    Complete(Child),
    Incomplete(FoldState),
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