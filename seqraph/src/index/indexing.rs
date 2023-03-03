use crate::*;
#[derive(Debug, Default)]
pub struct SplitCache {
    pub entries: HashMap<VertexIndex, SplitVertexCache>,
}
#[derive(Debug, Default)]
pub struct SplitVertexCache {
    pub positions: HashMap<TokenLocation, SplitPositionCache>,
}
#[derive(Debug, Default)]
pub struct SplitPositionCache {
    pub pattern_splits: HashMap<SubLocation, Split>,
    pub final_split: Option<Split>,
}
#[derive(Debug, Default, Clone)]
pub struct Split {
    left: Pattern,
    right: Pattern,
}
impl Split {
    pub fn infix(&mut self, mut inner: Split) {
        self.left.extend(inner.left);
        inner.right.extend(self.right.clone());
        self.right = inner.right;
    }
}
impl SplitCache {
    fn get(&self, key: &CacheKey) -> Option<&SplitPositionCache> {
        self.entries.get(&key.index.index())
            .and_then(|ve|
                ve.positions.get(&key.pos)
            )
    }
    fn get_mut(&mut self, key: &CacheKey) -> Option<&mut SplitPositionCache> {
        self.entries.get_mut(&key.index.index())
            .and_then(|ve|
                ve.positions.get_mut(&key.pos)
            )
    }
    fn expect_mut(&mut self, key: &CacheKey) -> &mut SplitPositionCache {
        self.get_mut(key).unwrap()
    }
    fn expect(&self, key: &CacheKey) -> &SplitPositionCache {
        self.get(key).unwrap()
    }
    fn get_split(&self, key: &CacheKey) -> Option<&Split> {
        self.get(key)
            .and_then(|e|
                e.final_split.as_ref()
            )
    }
    fn get_joined_splits(&mut self, indexer: &mut Indexer, key: &CacheKey) -> Split {
        if let Some(split) = self.get_split(key) {
            split.clone()
        } else {
            self.join_splits(indexer, key)
        }
    }
    fn join_splits(&mut self, indexer: &mut Indexer, key: &CacheKey) -> Split {
        let entry = self.expect_mut(key);
        let (l, r): (Vec<_>, Vec<_>) = entry.pattern_splits
            .drain()
            .map(|(_, s)| (s.left, s.right))
            .unzip();
        let mut graph = indexer.graph_mut();
        let lc = graph.insert_patterns(l);
        let rc = graph.insert_patterns(r);
        graph.add_pattern_with_update(&key.index, vec![lc, rc]);
        entry.final_split = Some(Split {
            left: vec![lc],
            right: vec![rc],
        });
        entry.final_split.as_ref().unwrap().clone()
    }
    fn add(&mut self, key: &CacheKey, location: SubLocation, split: Split) {
        self.expect_mut(key)
            .pattern_splits
            .insert(location, split);
    }
}
impl Indexer {
    pub fn index_leaf<R: PathRole>(
        &mut self,
        subgraph: &FoldState,
        splits: &mut SplitCache,
        frontier: &mut VecDeque<CacheKey>,
        key: &CacheKey,
    ) {
        let (l, r) = R::directed_pattern_split(&[key.index], 0);
        splits.expect_mut(key).final_split = Some(Split {
            left: l,
            right: r,
        });
        frontier.extend(subgraph.cache.expect(key).edges.top.keys());
    }
    pub fn index_inner_node(
        &mut self,
        subgraph: &FoldState,
        splits: &mut SplitCache,
        frontier: &mut VecDeque<CacheKey>,
        key: &CacheKey,
    ) {
        let entry = subgraph.cache.expect(&key);
        let inner_split = splits.get_joined_splits(self, &key);
        let graph = self.graph();
        for child_key in &entry.edges.bottom {
            let location = subgraph.cache.expect(&child_key).edges.top.get(key).unwrap();
            let pattern = graph.expect_pattern_at(&key.index.to_pattern_location(location.pattern_id));
            let (left, right) = split_context(pattern.borrow(), location.sub_index);
            let mut s = Split { left, right };
            s.infix(inner_split.clone());
            splits.add(&key, *location, s);
        }
        frontier.extend(subgraph.cache.expect(key).edges.top.keys());
    }
    pub fn index_subgraph(
        &mut self,
        subgraph: FoldState,
    ) -> Child {
        let roots = subgraph.roots();
        let root = roots
            .iter()
            .min_by(|a, b| a.index.width().cmp(&b.index.width()))
            .unwrap();
        let leaves = subgraph.leaves(&root);
        let mut frontier: VecDeque<_> = Default::default();
        let mut splits = SplitCache::default();
        self.index_leaf::<Start>(&subgraph, &mut splits, &mut frontier, &subgraph.start);
        for key in &leaves {
            self.index_leaf::<End>(&subgraph, &mut splits, &mut frontier, &key);
        }
        while let Some(key) = frontier.pop_front() {
            self.index_inner_node(
                &subgraph,
                &mut splits,
                &mut frontier,
                &key,
            );
        }
        unimplemented!();
    }
    ////pub fn index_subgraph<R: ResultKind, Q: QueryPath>(
    ////    &mut self,
    ////    result: FoldResult<R, Q>
    ////) -> Child {
    ////    let root_index = result.root_index();
    ////    let root_entries = result.root_entries()
    ////}
    //fn index_prefix_path(
    //    &mut self,
    //    path: RootedRolePath<End>,
    //) -> Child {
    //    self.splitter::<IndexFront>().single_path_split(
    //        std::iter::once(&path.child_location()).chain(
    //            path.path().into_iter()
    //        ).collect_vec(),
    //    )
    //    
    //    .map(|split| split.inner)
    //    .expect("RolePath for complete path!")
    //}
    //fn at_postfix_path(
    //    &mut self,
    //    path: RootedRolePath<Start>,
    //) -> Child {
    //    self.splitter::<IndexBack>().single_path_split(
    //        path.path().into_iter().chain(
    //            std::iter::once(&path.child_location())
    //        ).collect_vec(),
    //    )
    //    
    //    .map(|split| split.inner)
    //    .expect("RolePath for complete path!")
    //}
    //#[instrument(skip(self, path))]
    //fn index_range_path(
    //    &mut self,
    //    path: SearchPath,
    //) -> Child {
    //    //
    //    let entry = path.start.child_location();
    //    let entry_pos = RootChildPos::<Start>::root_child_pos(&path);
    //    let exit_pos = RootChildPos::<End>::root_child_pos(&path);

    //    let location = entry.into_pattern_location();

    //    //
    //    let range = G::Direction::wrapper_range(entry_pos, exit_pos);
    //    //self.graph().validate_pattern_indexing_range_at(&location, entry_pos, exit_pos).unwrap();

    //    let inserted = self.graph_mut().insert_range_in(
    //            location,
    //            range,
    //        );
    //    //
    //    let (wrapper, pattern, location) = if let Ok(wrapper) = inserted {
    //        let (pid, pattern) = self.graph().expect_any_child_pattern(wrapper)
    //            .pipe(|(&pid, pattern)|
    //                (pid, pattern.clone())
    //            );
    //        let location = wrapper.to_pattern_location(pid);
    //        (wrapper, pattern, location)
    //    } else {
    //        let wrapper = location.parent;
    //        let pattern = self.graph().expect_child_pattern(wrapper, location.pattern_id).clone();
    //        (wrapper, pattern, location)
    //    };

    //    let head_pos = G::Direction::head_index(pattern.borrow());
    //    let last_pos = G::Direction::last_index(pattern.borrow());

    //    let mut head_contexter = self.contexter::<IndexBack>();
    //    let head_split = self.splitter::<IndexBack>().single_path_split(
    //        path.start.path().to_vec()
    //    )
    //    .map(|split| (
    //        split.inner,
    //        head_contexter.try_context_path(
    //            split.path.into_iter().chain(
    //                std::iter::once(split.location)
    //            ).collect_vec(),
    //            //split.inner,
    //        ).unwrap().0
    //    ));

    //    let mut last_contexter = self.contexter::<IndexFront>();
    //    let last_split = 
    //        self.splitter::<IndexFront>().single_path_split(
    //            path.end.path().to_vec()
    //        )
    //        .map(|split| (
    //            split.inner,
    //            last_contexter.try_context_path(
    //                std::iter::once(split.location).chain(
    //                    split.path.into_iter()
    //                ).collect_vec(),
    //                //split.inner,
    //            ).unwrap().0
    //        ));

    //    let mut graph = self.graph_mut();
    //    let res = match (head_split, last_split) {
    //        (Some((head_inner, head_context)), Some((last_inner, last_context))) => {
    //            let range = G::Direction::inner_context_range(head_pos, last_pos);
    //            let inner = graph.insert_range_in(
    //                location,
    //                range,
    //            ).ok();
    //            let target = graph.insert_pattern(
    //                G::Direction::concat_context_inner_context(
    //                    head_inner,
    //                    inner.as_ref().map(std::slice::from_ref).unwrap_or_default(),
    //                    last_inner
    //                )
    //            );
    //            graph.add_pattern_with_update(
    //                wrapper,
    //                G::Direction::concat_context_inner_context(head_context, target, last_context)
    //            );
    //            target
    //        },
    //        (Some((head_inner, head_context)), None) => {
    //            let range = 
    //                <IndexBack as IndexSide<G::Direction>>::inner_context_range(head_pos);
    //            let inner_context = graph.insert_range_in_or_default(
    //                location,
    //                range,
    //            ).unwrap();
    //            // |context, [inner, inner_context]|
    //            let target = graph.insert_pattern(
    //                G::Direction::inner_then_context(head_inner, inner_context)
    //            );
    //            // |context, target|
    //            graph.add_pattern_with_update(
    //                wrapper,
    //                G::Direction::context_then_inner(head_context, target)
    //            );
    //            target
    //        },
    //        (None, Some((last_inner, last_context))) => {
    //            let range = 
    //                <IndexFront as IndexSide<G::Direction>>::inner_context_range(last_pos);
    //            let inner_context = graph.insert_range_in_or_default(
    //                location,
    //                range,
    //            ).unwrap();
    //            // |[inner_context, inner], context|
    //            let target = graph.insert_pattern(
    //                G::Direction::inner_then_context(inner_context, last_inner)
    //            );
    //            // |target, context|
    //            graph.add_pattern_with_update(
    //                wrapper,
    //                G::Direction::inner_then_context(target, last_context)
    //            );
    //            target
    //        },
    //        (None, None) => wrapper,
    //    };
    //    graph.validate_expansion(entry.parent);
    //    res
    //}
}