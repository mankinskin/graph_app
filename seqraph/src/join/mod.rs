use crate::*;

pub mod joined;
pub use joined::*;
pub mod delta;
pub use delta::*;
pub mod context;
pub use context::*;
pub mod partition;
pub use partition::*;

#[derive(Debug, Default, Deref, DerefMut)]
pub struct SplitFrontier {
    pub queue: LinkedHashSet<SplitKey>,
}
impl SplitFrontier {
    pub fn new(keys: impl IntoIterator<Item=SplitKey>) -> Self {
        Self {
            queue: LinkedHashSet::from_iter(keys),
        }
    }
}
impl Extend<SplitKey> for SplitFrontier {
    fn extend<T: IntoIterator<Item = SplitKey>>(&mut self, iter: T) {
        self.queue.extend(iter)
    }
}
impl Indexer {
    pub fn join_subgraph(
        &mut self,
        fold_state: FoldState,
    ) -> Child {
        let root = fold_state.root;
        let split_cache = SplitCache::new(self, fold_state);

        let mut frontier = SplitFrontier::new(
            split_cache.leaves.iter().cloned().rev()
        );
        let mut final_splits = HashMap::default();
        while let Some(key) = {
            frontier.pop_front()
                .and_then(|key|
                    (key.index != root).then(|| key)
                )
        } {
            if final_splits.get(&key).is_none() {
                let finals = {
                    let mut ctx = JoinContext::new(
                            self.graph_mut(),
                            &final_splits,
                        )
                        .node(key.index, &split_cache);
                    Self::join_node_partitions(&mut ctx)
                };

                for (key, split) in finals {
                    final_splits.insert(key, split);
                }
            }
            let top = 
                split_cache.expect(&key).top.iter()
                    .sorted_by(|a, b| a.index.width().cmp(&b.index.width()))
                    .cloned();
            frontier.extend(top);
        }
        let root_mode = split_cache.root_mode;
        let mut ctx = 
            JoinContext::new(
                    self.graph_mut(),
                    &final_splits,
                )
                .node(root, &split_cache);
        Self::join_root_partitions(
            &mut ctx,
            root_mode,
        )
    }
    pub fn index_partitions<'p, 't, S: HasPosSplits + 'p>(
        ctx: &mut NodeJoinContext<'p>,
        pos_splits: S,
    ) -> Vec<Child>
        where 'p: 't
    {
        let offset_splits = pos_splits.pos_splits();
        let len = offset_splits.len();
        assert!(len > 0);
        let mut iter = offset_splits.iter()
            .map(|(&offset, splits)|
                OffsetSplits {
                    offset,
                    splits: splits.borrow().clone(),
                }
            );
        let mut prev = iter.next().unwrap();
        let mut parts = Vec::with_capacity(1 + len);
        parts.push(
            Prefix::new(&prev).join_partition(ctx).into()
        );
        for offset in iter {
            parts.push(
                Infix::new(&prev, &offset).join_partition(ctx).into()
            );
            prev = offset;
        }
        parts.push(
            Postfix::new(prev).join_partition(ctx).into()
        );
        println!("{:#?}", parts);
        parts
    }
    pub fn join_node_partitions<'p>(
        ctx: &mut NodeJoinContext<'p>,
    ) -> LinkedHashMap<SplitKey, Split> {
        let partitions = Self::index_partitions(
            ctx,
            ctx.pos_splits,
        );
        assert_eq!(
            ctx.index.width(),
            partitions.iter().map(Child::width).sum::<usize>()
        );
        assert_eq!(
            partitions.len(),
            ctx.pos_splits.len() + 1,
        );
        ctx.merge_node(
            ctx.pos_splits,
            &partitions,
        )
    }
    pub fn join_root_partitions<'p>(
        ctx: &mut NodeJoinContext<'p>,
        root_mode: RootMode,
    ) -> Child {
        let index = ctx.index;
        let offsets = ctx.pos_splits;
        let num_offsets = offsets.len();
        let mut offset_iter = offsets.iter();
        let offset = offset_iter.next().unwrap();

        match root_mode {
            RootMode::Prefix => {
                assert_eq!(num_offsets, 1);
                Self::join_prefix_root(
                    ctx,
                    offset,
                    index,
                )
            },
            RootMode::Postfix => {
                assert_eq!(num_offsets, 1);
                Self::join_postfix_root(
                    ctx,
                    offset,
                    index,
                )
            },
            RootMode::Infix => {
                assert_eq!(num_offsets, 2);
                let prev_offset = offset;
                let offset = offset_iter.next().unwrap();

                Self::join_infix_root(
                    ctx,
                    prev_offset,
                    offset,
                    index,
                )
            }
        }
    }
    pub fn join_prefix_root<'p>(
        ctx: &mut NodeJoinContext<'p>,
        offset: PosSplitRef<'p>,
        index: Child,
    ) -> Child {
        match Prefix::new(offset).join_partition(ctx) {
            Ok(part) => {
                if let Some(pid) = part.perfect.into() {
                    let pos = &offset.1.pattern_splits[&pid];
                    ctx.graph.replace_in_pattern(
                        index.to_pattern_location(pid),
                        0..pos.sub_index,
                        [part.index],
                    )
                } else {
                    let post = Postfix::new(offset).join_partition(ctx).unwrap();
                    ctx.graph.add_pattern_with_update(
                        index,
                        [part.index, post.index],
                    );
                }
                part.index
            },
            Err(c) => c,
        }
    }
    pub fn join_postfix_root<'p>(
        ctx: &mut NodeJoinContext<'p>,
        offset: PosSplitRef<'p>,
        index: Child,
    ) -> Child {
        match Postfix::new(offset).join_partition(ctx) {
            Ok(part) => {
                println!("{:#?}", part);
                if let Some(pid) = part.perfect.into() {
                    let pos = &offset.1.pattern_splits[&pid];
                    ctx.graph.replace_in_pattern(
                        index.to_pattern_location(pid),
                        pos.sub_index..,
                        [part.index],
                    )
                } else {
                    let pre = match Prefix::new(offset).join_partition(ctx) {
                        Ok(pre) => {
                            println!("{:#?}", pre);
                            pre.index
                        },
                        Err(c) => c,
                    };
                    ctx.graph.add_pattern_with_update(
                        index,
                        [pre, part.index],
                    );
                }
                part.index
            },
            Err(c) => c,
        }
    }
    pub fn join_incomplete_infix<'p>(
        ctx: &mut NodeJoinContext<'p>,
        part: JoinedPartition<In<Join>>,
        prev_offset: PosSplitRef<'p>,
        offset: PosSplitRef<'p>,
        index: Child,
    ) -> Child {
        let mut prev_offset = (prev_offset.0, prev_offset.1.clone());
        let mut offset = (offset.0, offset.1.clone() - part.delta);

        if (None, None) == part.perfect.into() {
            // no perfect border
            //        [               ]
            // |     |      |      |     |   |
            let (offset, pre) = match Prefix::new(prev_offset).join_partition(ctx) {
                Ok(part) => 
                    (
                        (offset.0, (offset.1.clone() - part.delta)),
                        part.index,
                    ),
                Err(ch) =>
                    (
                        offset,
                        ch,
                    ),
            };
            let post: Child = Postfix::new(offset).join_partition(ctx).into();
            ctx.graph.add_pattern_with_update(
                index,
                [pre, part.index, post],
            );

        } else if part.perfect.0 == part.perfect.1 {
            // perfect borders in same pattern
            //       [               ]
            // |     |       |       |      |
            let (ll, rl) = (part.perfect.0.unwrap(), part.perfect.1.unwrap());
            let lpos = prev_offset.1.pattern_splits[&ll].sub_index;
            let rpos = offset.1.pattern_splits[&rl].sub_index;
            ctx.graph.replace_in_pattern(
                index.to_pattern_location(ll),
                lpos..rpos,
                [part.index],
            )
        } else {
            // one or both are perfect in different patterns
            if let Some(rp) = part.perfect.1 {
                //           [              ]
                // |     |       |     |    |     |

                // todo: improve syntax
                let pre = Prefix::new(prev_offset.clone()).join_partition(ctx).unwrap();
                prev_offset.1 = prev_offset.1 - pre.delta;

                let wrap_patterns = Prefix::new(offset.clone())
                    .info_partition(ctx).unwrap()
                    .to_joined_patterns(ctx);
                let patterns = wrap_patterns.patterns.clone();
                offset.1 = offset.1 - wrap_patterns.delta;
                let wrapper = ctx.graph.insert_patterns(
                    std::iter::once(vec![pre.index, part.index])
                        .chain(patterns),
                );

                let ri = offset.1.pattern_splits[&rp].sub_index;
                let loc = index.to_pattern_location(rp);
                ctx.graph.replace_in_pattern(
                    loc,
                    0..ri,
                    [wrapper],
                );
            }
            if let Some(lp) = part.perfect.0 {
                //       [                 ]
                // |     |       |      |      |   |
                let post = match Postfix::new(offset).info_partition(ctx) {
                    Ok(post_info) => {

                        post_info.to_joined_partition(ctx)
                    }
                    Err(post) => post,
                };

                let li = prev_offset.1.pattern_splits[&lp].sub_index;
                let part_info = Infix::new(prev_offset, )
                    .info_partition(ctx)
                    .unwrap();
                // todo: skip lp in part_info already
                //part_info.patterns.remove(&lp);
                let wrap_patterns = part_info.to_joined_patterns(ctx);

                let wrapper = ctx.graph.insert_patterns(
                    std::iter::once(vec![part.index, post.index])
                        .chain(wrap_patterns.patterns),
                );
                let loc = index.to_pattern_location(lp);
                ctx.graph.replace_in_pattern(
                    loc,
                    li..,
                    [wrapper],
                );
            }
        }
        part.index
    }
    pub fn join_infix_root<'p>(
        ctx: &mut NodeJoinContext<'p>,
        prev_offset: PosSplitRef<'p>,
        offset: PosSplitRef<'p>,
        index: Child,
    ) -> Child {
        match Infix::new(prev_offset, offset).join_partition(ctx) {
            Ok(part) => Self::join_incomplete_infix(
                ctx,
                part,
                prev_offset,
                offset,
                index,
            ),
            Err(c) => c,
        }
    }
}