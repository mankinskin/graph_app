use crate::*;
pub mod vertex;
pub use vertex::*;
pub mod split;
pub use split::*;

#[derive(Default, Debug, Deref, DerefMut, From)]
pub struct Leaves(Vec<SplitKey>);
impl Leaves {
    pub fn filter_leaves(&mut self, index: &Child, offsets: CompleteLocations) -> HashMap<Offset, Vec<SubSplitLocation>> {
        offsets.into_iter()
            .filter_map(|(parent_offset, res)|
                match res {
                    Ok(locs) => Some((parent_offset, locs)),
                    Err(_) => {
                        self.push(SplitKey::new(*index, parent_offset));
                        None
                    },
                }
            )
            .collect()
    }
    /// kind of like filter_leaves but from subsplits to trace states
    pub fn filter_trace_states<Trav: Traversable>(
        &mut self,
        trav: &Trav,
        index: &Child,
        sub_splits: impl IntoIterator<Item=(Offset, Vec<SubSplitLocation>)>,
    ) -> Vec<TraceState> {
        let graph = trav.graph();
        let (_, node) = graph.expect_vertex(index);
        sub_splits.into_iter()
            .flat_map(|(parent_offset, locs)| {
                let len = locs.len();
                locs.into_iter()
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
                                self.push(SplitKey::new(*index, parent_offset));
                            }
                            None
                        })
                    })
            }).collect()
    }
}
#[derive(Debug)]
pub struct SplitCache {
    pub entries: HashMap<VertexIndex, SplitVertexCache>,
    pub leaves: Leaves,
    pub states: VecDeque<TraceState>,
    pub root_mode: RootMode,
}
impl SplitCache {
    pub fn new<Trav: Traversable>(
        fold_state: &mut FoldState,
        trav: &Trav
    ) -> Self {
        let mut states = VecDeque::default();
        let mut entries = HashMap::default();
        let mut leaves = Leaves::default();
        let (root_vertex, root_mode) = Self::new_root_vertex(
            trav,
            &mut states,
            fold_state,
            &mut leaves,
        );
        entries.insert(
            fold_state.root.index(),
            root_vertex,
        );
        SplitCache {
            entries,
            leaves,
            root_mode,
            states,
        }
    }
    pub fn trace<Trav: Traversable>(
        &mut self,
        trav: &Trav,
        fold_state: &mut FoldState,
        state: TraceState,
    ) {
        let TraceState { index, offset, prev } = state;
        if let Some(ve) = self.entries.get_mut(&index.index()) {
            ve.positions.entry(offset)
                .and_modify(|pe| {
                    pe.top.insert(prev);
                })
                .or_insert_with(||
                    self.new_split_position(
                        trav,
                        index,
                        offset,
                        prev,
                    )
                );
        } else {
            let vertex = self.new_split_vertex(
                trav,
                index,
                offset,
                prev,
                fold_state,
            );
            self.entries.insert(
                index.index(),
                vertex,
            );
        }
    }
    pub fn complete_node<'a>(
        &mut self,
        ctx: JoinContext<'a>,
        fold_state: &mut FoldState,
        prev: SplitKey,
    ) -> Vec<TraceState> {
        self.entries.get_mut(&ctx.index.index()).unwrap().complete_node(
            ctx,
            prev,
        )
    }
    pub fn new_root_vertex<Trav: Traversable>(
        trav: &Trav,
        states: &mut VecDeque<TraceState>,
        fold_state: &FoldState,
        leaves: &mut Leaves,
    ) -> (SplitVertexCache, RootMode) {
        let (offsets, root_mode) = fold_state.complete_splits::<_, RootNode>(
            trav,
            &fold_state.root,
        );
        states.extend(leaves.filter_trace_states(
            trav,
            &fold_state.root,
            leaves.filter_leaves(&fold_state.root, offsets.clone()),
        ));
        (
            SplitVertexCache {
                positions: offsets.into_iter().map(|(offset, res)|
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
    pub fn new_split_vertex<Trav: Traversable>(
        &mut self,
        trav: &Trav,
        index: Child,
        offset: NonZeroUsize,
        prev: SplitKey,
        fold_state: &mut FoldState,
    ) -> SplitVertexCache {
        let mut subs = fold_state.complete_splits::<_, InnerNode>(
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
        self.states.extend(self.leaves.filter_trace_states(
            trav,
            &index,
            self.leaves.filter_leaves(&index, subs.clone()),
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
    pub fn new_split_position<Trav: Traversable>(
        &mut self,
        trav: &Trav,
        index: Child,
        offset: NonZeroUsize,
        prev: SplitKey,
        //states: &mut VecDeque<TraceState>,
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
                self.states.extend(self.leaves.filter_trace_states(
                    trav,
                    &index,
                    HashMap::from_iter([(offset, subs.clone())]),
                ));
                SplitPositionCache::new(prev, subs)
            },
            Err(location) => {
                self.leaves.push(SplitKey::new(index, offset));
                SplitPositionCache::new(prev, vec![
                    SubSplitLocation {
                        location,
                        inner_offset: None,
                    }
                ])
            }
        }
    }
    pub fn get(&self, key: &SplitKey) -> Option<&SplitPositionCache> {
        self.entries.get(&key.index.index())
            .and_then(|ve|
                ve.positions.get(&key.pos)
            )
    }
    pub fn get_mut(&mut self, key: &SplitKey) -> Option<&mut SplitPositionCache> {
        self.entries.get_mut(&key.index.index())
            .and_then(|ve|
                ve.positions.get_mut(&key.pos)
            )
    }
    pub fn expect(&self, key: &SplitKey) -> &SplitPositionCache {
        self.get(key).unwrap()
    }
    pub fn expect_mut(&mut self, key: &SplitKey) -> &mut SplitPositionCache {
        self.get_mut(key).unwrap()
    }
    pub fn get_final_split(&self, key: &SplitKey) -> Option<&FinalSplit> {
        self.get(key)
            .and_then(|e|
                e.final_split.as_ref()
            )
    }
    pub fn expect_final_split(&self, key: &SplitKey) -> &FinalSplit {
        self.expect(key).final_split.as_ref().unwrap()
    }
}

pub fn position_splits<'a>(
    patterns: impl Iterator<Item=(&'a PatternId, &'a Pattern)>,
    offset: NonZeroUsize,
) -> OffsetSplits {
    OffsetSplits {
        offset,
        splits: patterns
        .map(|(pid, pat)| { 
            let (sub_index, inner_offset) = <IndexBack as IndexSide<Right>>::token_offset_split(
                pat.borrow() as &[Child],
                offset,
            ).unwrap();
            (*pid, PatternSplitPos {
                sub_index,
                inner_offset,
            })
        })
        .collect(),
    }
}
pub fn range_splits<'a>(
    patterns: impl Iterator<Item=(&'a PatternId, &'a Pattern)>,
    parent_range: (NonZeroUsize, NonZeroUsize),
) -> (OffsetSplits, OffsetSplits) {
    let (ls, rs) = patterns
        .map(|(pid, pat)| {
            let (li, lo) = <IndexBack as IndexSide<Right>>::token_offset_split(
                pat.borrow() as &[Child],
                parent_range.0,
            ).unwrap();
            let (ri, ro) = <IndexBack as IndexSide<Right>>::token_offset_split(
                pat.borrow() as &[Child],
                parent_range.1,
            ).unwrap();
            (
                (
                    *pid,
                    PatternSplitPos {
                        sub_index: li,
                        inner_offset: lo,
                    }
                ),
                (
                    *pid,
                    PatternSplitPos {
                        sub_index: ri,
                        inner_offset: ro,
                    }
                ),
            )
        })
        .unzip();
    (
        OffsetSplits {
            offset: parent_range.0,
            splits: ls,
        },
        OffsetSplits {
            offset: parent_range.1,
            splits: rs,
        },
    )
}

pub fn cleaned_position_splits<'a>(
    patterns: impl Iterator<Item=(&'a PatternId, &'a Pattern)>,
    parent_offset: NonZeroUsize,
) -> Result<Vec<SubSplitLocation>, SubLocation> {
    patterns
        .map(|(pid, pat)| { 
            let (sub_index, inner_offset) = <IndexBack as IndexSide<Right>>::token_offset_split(
                pat.borrow() as &[Child],
                parent_offset,
            ).unwrap();
            let location = SubLocation::new(*pid, sub_index);
            if inner_offset.is_some() || pat.len() > 2 {
                // can't be clean
                Ok(SubSplitLocation {
                    location,
                    inner_offset
                })
            } else {
                // must be clean
                Err(location)
            }
        })
        .collect()
}