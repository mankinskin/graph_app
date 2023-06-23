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
        split_pos: impl IntoIterator<Item=(Offset, Vec<SubSplitLocation>)>,
    ) -> Vec<TraceState> {
        let graph = trav.graph();
        let (_, node) = graph.expect_vertex(index);
        let (perfect, next) = split_pos.into_iter()
            .flat_map(|(parent_offset, locs)| {
                let len = locs.len();
                locs.into_iter()
                    .map(move |sub|
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
                        ).ok_or_else(||
                            (len == 1).then(||
                                SplitKey::new(*index, parent_offset)
                            )
                        )
                    )
            })
            .fold((Vec::new(), Vec::new()), |(mut p, mut n), res| {
                match res {
                    Ok(s) => n.push(s),
                    Err(Some(k)) => p.push(k),
                    Err(None) => {}
                }
                (p, n)
            });
        self.extend(perfect);
        next
    }
}
#[derive(Debug, Deref, DerefMut)]
pub struct SplitCache {
    pub entries: HashMap<VertexIndex, SplitVertexCache>,
    #[deref]
    #[deref_mut]
    pub context: CacheContext,
    pub root_mode: RootMode,
}
#[derive(Debug)]
pub struct CacheContext {
    pub leaves: Leaves,
    pub states: VecDeque<TraceState>,
}
impl CacheContext {
    pub fn new_split_position<Trav: Traversable>(
        &mut self,
        trav: &Trav,
        index: Child,
        offset: NonZeroUsize,
        prev: SplitKey,
    ) -> SplitPositionCache {
        let graph = trav.graph();
        let (_, node) = graph.expect_vertex(index);

        // handle clean splits
        match cleaned_position_splits(
            node.children.iter(),
            offset,
        ) {
            Ok(subs) => {
                let next = self.leaves.filter_trace_states(
                    trav,
                    &index,
                    HashMap::from_iter([(offset, subs.clone())]),
                );
                self.states.extend(next);
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
            root_mode,
            context: CacheContext {
                leaves,
                states,
            }
        }
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
        let split_pos = leaves.filter_leaves(&fold_state.root, offsets.clone());
        states.extend(leaves.filter_trace_states(
            trav,
            &fold_state.root,
            split_pos
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
        let split_pos = self.leaves.filter_leaves(&index, subs.clone());
        let next = self.leaves.filter_trace_states(
            trav,
            &index,
            split_pos,
        );
        self.states.extend(next);
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
    /// complete offsets across all children
    pub fn trace<Trav: Traversable>(
        &mut self,
        trav: &Trav,
        fold_state: &mut FoldState,
        state: &TraceState,
    ) {
        let &TraceState { index, offset, prev } = state;
        if let Some(ve) = self.entries.get_mut(&index.index()) {
            let ctx = &mut self.context;
            ve.positions.entry(offset)
                .and_modify(|pe| {
                    pe.top.insert(prev);
                })
                .or_insert_with(||
                    ctx.new_split_position(
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
    /// complete inner range offsets for non-roots
    pub fn complete_node<'a>(
        &mut self,
        ctx: TraceContext<'a>,
    ) -> Vec<TraceState> {
        self.entries.get_mut(&ctx.index.index()).unwrap()
            .complete_node(
                ctx,
            )
    }
    /// complete inner range offsets for root
    pub fn complete_root<'a>(
        &mut self,
        ctx: TraceContext<'a>,
        root_mode: RootMode,
    ) -> Vec<TraceState> {
        self.entries.get_mut(&ctx.index.index()).unwrap()
            .complete_root(
                ctx,
                root_mode,
            )
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
    //pub fn get_final_split(&self, key: &SplitKey) -> Option<&FinalSplit> {
    //    self.get(key)
    //        .and_then(|e|
    //            e.final_split.as_ref()
    //        )
    //}
    //pub fn expect_final_split(&self, key: &SplitKey) -> &FinalSplit {
    //    self.expect(key).final_split.as_ref().unwrap()
    //}
}

pub fn position_splits<'a>(
    patterns: impl IntoIterator<Item=(&'a PatternId, &'a Pattern)>,
    offset: NonZeroUsize,
) -> OffsetSplits {
    OffsetSplits {
        offset,
        splits: patterns.into_iter()
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