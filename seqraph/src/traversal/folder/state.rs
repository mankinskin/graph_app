use crate::*;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct FoldState {
    pub cache: TraversalCache,
    pub end_states: Vec<EndState>,
    pub(crate) start: Child,
    pub(crate) root: Child,
    pub(crate) end_pos: TokenLocation,
}
#[derive(Debug, PartialEq, Eq, Clone)]
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
    pub fn add_split_position<Trav: Traversable>(
        &mut self,
        trav: &Trav,
        index: Child,
        offset: NonZeroUsize,
        prev: SplitKey,
        states: &mut VecDeque<TraceState>,
        leaves: &mut Vec<SplitKey>,
    ) -> SplitPositionCache {
        let graph = trav.graph();
        let (_, node) = graph.expect_vertex(index);
        //let entry = self.cache.entries.get(&index.index).unwrap();

        // handle clean splits
        match cleaned_position_splits(
            node.children.iter(),
            offset,
        ) {
            Ok(subs) => {
                states.extend(self.to_trace_states(
                    trav,
                    &index,
                    HashMap::from_iter([(offset, subs.clone())]),
                    leaves,
                ));
                SplitPositionCache::new(prev, subs)
            },
            Err(location) => {
                leaves.push(SplitKey::new(index, offset));
                SplitPositionCache::new(prev, vec![
                    SubSplitLocation {
                        location,
                        inner_offset: None,
                    }
                ])
            }
        }
    }
    pub fn add_split_vertex<Trav: Traversable>(
        &mut self,
        trav: &Trav,
        index: Child,
        offset: NonZeroUsize,
        prev: SplitKey,
        states: &mut VecDeque<TraceState>,
        leaves: &mut Vec<SplitKey>,
    ) -> SplitVertexCache {
        let mut subs = self.complete_splits::<_, InnerNode>(
            trav,
            &index,
        );
        if subs.get(&offset).is_none() {
            let graph = trav.graph();
            let (_, node) = graph.expect_vertex(index);
            //let entry = self.cache.entries.get(&index.index).unwrap();
            subs.insert(offset,
                cleaned_position_splits(
                    node.children.iter(),
                    offset,
                )
            );
        }
        states.extend(self.to_trace_states(
            trav,
            &index,
            subs.clone()
                .into_iter()
                .filter_map(|(parent_offset, res)|
                    match res {
                        Ok(locs) => Some((parent_offset, locs)),
                        Err(_) => {
                            leaves.push(SplitKey::new(index, parent_offset));
                            None
                        },
                    }
                )
                .collect_vec(),
            leaves,
        ));

        SplitVertexCache {
            positions: subs.into_iter().map(|(offset, res)|
                (offset, SplitPositionCache::new(
                    prev,
                    res.unwrap_or_else(|location|
                        vec![
                            SubSplitLocation {
                                location,
                                inner_offset: None,
                            }
                        ]
                    )
                ))
            ).collect()
        }
    }
    pub fn add_root_vertex<Trav: Traversable>(
        &mut self,
        trav: &Trav,
        index: Child,
        states: &mut VecDeque<TraceState>,
        leaves: &mut Vec<SplitKey>,
    ) -> (SplitVertexCache, RootMode) {
        let (subs, root_mode) = self.complete_splits::<_, RootNode>(
            trav,
            &index,
        );
        states.extend(self.to_trace_states(
            trav,
            &index,
            subs.clone()
                .into_iter()
                .filter_map(|(parent_offset, res)|
                    match res {
                        Ok(locs) => Some((parent_offset, locs)),
                        Err(_) => {
                            leaves.push(SplitKey::new(index, parent_offset));
                            None
                        },
                    }
                )
                .collect_vec(),
            leaves,
        ));
        (
            SplitVertexCache {
                positions: subs.into_iter().map(|(offset, res)|
                    (offset, SplitPositionCache::root(
                        res.unwrap_or_else(|location|
                            vec![
                                SubSplitLocation {
                                    location,
                                    inner_offset: None,
                                }
                            ]
                        )
                    ))
                ).collect()
            },
            root_mode,
        )
    }
    pub fn into_split_graph<Trav: Traversable>(
        &mut self,
        trav: &Trav,
    ) -> SplitCache {
        let mut states = VecDeque::default();
        let mut entries = HashMap::default();
        let mut leaves = Vec::default();

        let (root_vertex, root_mode) = self.add_root_vertex(
            trav,
            self.root,
            &mut states,
            &mut leaves,
        );
        entries.insert(
            self.root.index(),
            root_vertex,
        );
        while let Some(TraceState { index, offset, prev }) = states.pop_front() {
            if let Some(ve) = entries.get_mut(&index.index()) {
                ve.positions.entry(offset)
                    .and_modify(|pe| {
                        pe.top.insert(prev);
                    })
                    .or_insert_with(||
                        self.add_split_position(
                            trav,
                            index,
                            offset,
                            prev,
                            &mut states,
                            &mut leaves
                        )
                    );
            } else {
                entries.insert(
                    index.index(),
                    self.add_split_vertex(
                        trav,
                        index,
                        offset,
                        prev,
                        &mut states,
                        &mut leaves,
                    )
                );
            }
        };
        SplitCache {
            entries,
            leaves,
            root_mode,
        }
    }
    pub fn to_trace_states<Trav: Traversable>(
        &mut self,
        trav: &Trav,
        index: &Child,
        sub_splits: impl IntoIterator<Item=(Offset, Vec<SubSplitLocation>)>,
        leaves: &mut Vec<SplitKey>,
    ) -> Vec<TraceState> {
        let graph = trav.graph();
        let (_, node) = graph.expect_vertex(index);
        let (states, new_leaves) = sub_splits.into_iter()
            .fold((vec![], vec![]), |(mut states, mut leaves), (parent_offset, locs)| {
                let len = locs.len();
                states.extend(locs.into_iter()
                    .flat_map(|sub| {
                        // filter sub locations without offset (perfect splits)
                        sub.inner_offset.map(|offset|
                            TraceState {
                                index: *node.expect_child_at(&sub.location),
                                offset,
                                prev: SplitKey {
                                    index: *index,
                                    pos: parent_offset,
                                }
                            }
                        ).or_else(|| {
                            if len == 1 {
                                leaves.push(SplitKey::new(*index, parent_offset));
                            }
                            None
                        })
                    })
                );
                (states, leaves)
            });
        leaves.extend(new_leaves);
        states
    }
    pub fn complete_splits<Trav: Traversable, N: NodeType>(
        &mut self,
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
        leaves: &mut Vec<SplitKey>,
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
        self.to_trace_states(
            trav,
            index,
            subs,
            leaves
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

#[derive(Debug, Clone)]
pub struct TraceState {
    index: Child,
    offset: NonZeroUsize,
    prev: SplitKey,
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