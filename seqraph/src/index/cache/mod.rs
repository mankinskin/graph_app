use crate::*;
pub mod vertex;
pub use vertex::*;
pub mod split;
pub use split::*;
pub mod leaves;
pub use leaves::*;

#[derive(Debug, Clone)]
pub struct TraceState {
    pub index: Child,
    pub offset: NonZeroUsize,
    pub prev: SplitKey,
}

#[derive(Debug, Deref, DerefMut)]
pub struct SplitCache {
    pub entries: HashMap<VertexIndex, SplitVertexCache>,
    #[deref]
    #[deref_mut]
    pub context: CacheContext,
    pub root_mode: RootMode,
}
impl SplitCache {
    pub fn new<'a, Trav: TraversableMut<GuardMut<'a> = RwLockWriteGuard<'a, Hypergraph>> + 'a>(
        trav: &'a mut Trav,
        mut fold_state: FoldState,
    ) -> Self {
        let mut states = VecDeque::default();
        let mut entries = HashMap::default();
        let mut leaves = Leaves::default();
        let (root_vertex, root_mode) = Self::new_root_vertex(
            trav,
            &fold_state,
            &mut states,
            &mut leaves,
        );
        entries.insert(
            fold_state.root.index(),
            root_vertex,
        );
        let mut cache = Self {
            entries,
            root_mode,
            context: CacheContext {
                leaves,
                states,
            }
        };
        let graph = trav.graph_mut();
        cache.complete_root(
            TraceContext::new(
                &graph,
                fold_state.root,
            ),
            fold_state.root_mode(),
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
            cache.trace(
                &graph,
                &mut fold_state,
                &state,
            );
            incomplete.insert(state.index);
            let complete = incomplete.split_off(&ChildWidth(state.index.width() + 1));
            cache.complete_nodes(
                &graph,
                complete,
            );
        };
        cache.complete_nodes(
            &graph,
            incomplete,
        );
        cache
    }
    pub fn completed_splits<Trav: Traversable, N: NodeType>(
        trav: &Trav,
        fold_state: &FoldState,
        index: &Child,
    ) -> N::CompleteSplitOutput {
        fold_state.cache.entries.get(&index.index()).map(|e|
            e.complete_splits::<_, N>(
                trav,
                fold_state.end_pos,
            )
        )
        .unwrap_or_default()
    }
    pub fn child_trace_states<Trav: Traversable>(
        &mut self,
        trav: &Trav,
        fold_state: &FoldState,
        index: &Child,
    ) -> Vec<TraceState> {
        let subs =
            Self::completed_splits::<_, InnerNode>(
                trav,
                fold_state,
                index,
            )
            .into_iter()
            .filter_map(|(parent_offset, res)|
                res.ok().map(|locs|
                    (parent_offset, locs)
                )
            );
        self.leaves.filter_trace_states(
            trav,
            index,
            subs,
        )
    }
    pub fn new_root_vertex<Trav: Traversable>(
        trav: &Trav,
        fold_state: &FoldState,
        states: &mut VecDeque<TraceState>,
        leaves: &mut Leaves,
    ) -> (SplitVertexCache, RootMode) {
        let (offsets, root_mode) = Self::completed_splits::<_, RootNode>(
            trav,
            fold_state,
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
        fold_state: &mut FoldState,
        index: Child,
        offset: NonZeroUsize,
        prev: SplitKey,
    ) -> SplitVertexCache {
        let mut subs = Self::completed_splits::<_, InnerNode>(
            trav,
            fold_state,
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
                fold_state,
                index,
                offset,
                prev,
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
    pub fn complete_nodes<
        I: IntoIterator<Item=Child>,
    >(
        &mut self,
        graph: &RwLockWriteGuard<'_, Hypergraph>,
        nodes: I,
    ) {
        for c in nodes {
            let new = self.complete_node(
                TraceContext::new(
                    graph,
                    c,
                ),
            );
            // todo: force order
            self.states.extend(new.into_iter());
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

pub fn position_splits<'a>(
    patterns: impl IntoIterator<Item=(&'a PatternId, &'a Pattern)>,
    offset: NonZeroUsize,
) -> OffsetSplits {
    OffsetSplits {
        offset,
        splits: patterns.into_iter()
            .map(|(pid, pat)| { 
                let (sub_index, inner_offset) = IndexBack::token_offset_split(
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
            let (li, lo) = IndexBack::token_offset_split(
                pat.borrow() as &[Child],
                parent_range.0,
            ).unwrap();
            let (ri, ro) = IndexBack::token_offset_split(
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
            let (sub_index, inner_offset) = IndexBack::token_offset_split(
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
#[cfg(test)]
mod tests {
    use crate::*;
    use pretty_assertions::assert_eq;
    #[test]
    fn split_graph1() {
        let Context {
            graph,
            a,
            d,
            e,
            bc,
            abc,
            abcd,
            abc_d_id,
            abcdef,
            //abc_def_id,
            abcd_ef_id,
            //def,
            ef,
            e_f_id,
            ..
        } = &mut *context_mut();
        let query = vec![*a, *bc, *d, *e];
        let res = graph.searcher().find_pattern_ancestor(query)
            .unwrap().result
            .unwrap_incomplete();
        assert_eq!(res.start, *a);
        assert_eq!(res.end_pos, 5.into());
        assert_eq!(res.cache.entries, HashMap::from_iter([
            (a.index, VertexCache {
                index: *a,
                bottom_up: HashMap::from_iter([
                    (1.into(), PositionCache {
                        edges: Default::default(),
                        index: *a,
                        waiting: Default::default(),
                    })
                ]),
                top_down: HashMap::from_iter([]),
            }),
            (bc.index, VertexCache {
                index: *bc,
                bottom_up: HashMap::from_iter([]),
                top_down: HashMap::from_iter([
                    (1.into(), PositionCache {
                        edges: Default::default(),
                        index: *bc,
                        waiting: Default::default(),
                    })
                ]),
            }),
            (abc.index, VertexCache {
                index: *abc,
                bottom_up: HashMap::from_iter([
                    (1.into(), PositionCache {
                        edges: Edges {
                            top: Default::default(),
                            bottom: HashMap::from_iter([
                                (
                                    DirectedKey::up(*a, 1),
                                    SubLocation::new(3, 0),
                                )
                            ]),
                        },
                        index: *abc,
                        waiting: Default::default(),
                    })
                ]),
                top_down: HashMap::from_iter([]),
            }),
            (abcd.index, VertexCache {
                index: *abcd,
                bottom_up: HashMap::from_iter([
                    (3.into(), PositionCache {
                        edges: Edges {
                            top: Default::default(),
                            bottom: HashMap::from_iter([
                                (
                                    DirectedKey::up(*abc, 3),
                                    SubLocation::new(*abc_d_id, 0),
                                )
                            ]),
                        },
                        index: *abcd,
                        waiting: Default::default(),
                    })
                ]),
                top_down: HashMap::from_iter([]),
            }),
            (abcdef.index, VertexCache {
                index: *abcdef,
                bottom_up: HashMap::from_iter([
                    (4.into(), PositionCache {
                        edges: Edges {
                            top: HashSet::from_iter([]),
                            bottom: HashMap::from_iter([
                                //(
                                //    DirectedKey::up(*abc, 3),
                                //    SubLocation::new(*abc_def_id, 0),
                                //),
                                //(
                                //    DirectedKey::down(*def, 3),
                                //    SubLocation::new(*abc_def_id, 1),
                                //),
                                (
                                    DirectedKey::up(*abcd, 4),
                                    SubLocation::new(*abcd_ef_id, 0),
                                ),
                                (
                                    DirectedKey::down(*ef, 4),
                                    SubLocation::new(*abcd_ef_id, 1),
                                ),
                            ]),
                        },
                        index: *abcdef,
                        waiting: Default::default(),
                    })
                ]),
                top_down: HashMap::from_iter([
                    (4.into(), PositionCache {
                        edges: Edges {
                            top: HashSet::from_iter([]),
                            bottom: HashMap::from_iter([
                                (
                                    DirectedKey::down(*ef, 4),
                                    SubLocation::new(*abcd_ef_id, 1),
                                )
                            ]),
                        },
                        index: *abcdef,
                        waiting: Default::default(),
                    })
                ]),
            }),
            (ef.index, VertexCache {
                index: *ef,
                bottom_up: HashMap::from_iter([]),
                top_down: HashMap::from_iter([
                    (4.into(), PositionCache {
                        edges: Edges {
                            top: HashSet::from_iter([]),
                            bottom: HashMap::from_iter([
                                (
                                    DirectedKey::down(*e, 4),
                                    SubLocation::new(*e_f_id, 0),
                                )
                            ]),
                        },
                        index: *ef,
                        waiting: Default::default(),
                    })
                ]),
            }),
            (e.index, VertexCache {
                index: *e,
                top_down: HashMap::from_iter([
                    (4.into(), PositionCache {
                        edges: Default::default(),
                        index: *e,
                        waiting: Default::default(),
                    })
                ]),
                bottom_up: HashMap::from_iter([]),
            }),
            (d.index, VertexCache {
                index: *d,
                top_down: HashMap::from_iter([
                    (3.into(), PositionCache {
                        edges: Default::default(),
                        index: *d,
                        waiting: Default::default(),
                    })
                ]),
                bottom_up: HashMap::from_iter([]),
            }),
        ]));
        let splits = SplitCache::new(
            &mut *graph,
            res,
        );
        assert_eq!(
            splits.entries,
            HashMap::from_iter([
                (a.index, SplitVertexCache {
                    positions: BTreeMap::default(),
                }),
                (bc.index, SplitVertexCache {
                    positions: BTreeMap::from_iter([
                        (1.try_into().unwrap(), SplitPositionCache {
                            top: HashSet::from_iter([
                                SplitKey::new(*abc, 3),
                            ]),
                            pattern_splits: HashMap::from_iter([]),
                        })
                    ]),
                }),
                (abc.index, SplitVertexCache {
                    positions: BTreeMap::from_iter([
                    ]),
                }),
                (abcd.index, SplitVertexCache {
                    positions: BTreeMap::from_iter([
                    ]),
                }),
                (abcdef.index, SplitVertexCache {
                    positions: BTreeMap::from_iter([
                    ]),
                }),
                (ef.index, SplitVertexCache {
                    positions: BTreeMap::from_iter([
                    ]),
                }),
                (e.index, SplitVertexCache {
                    positions: BTreeMap::default(),
                }),
                (d.index, SplitVertexCache {
                    positions: BTreeMap::default(),
                }),
            ])
        )
    }
}