use std::{
    borrow::Borrow,
    iter::FromIterator,
    num::NonZeroUsize,
};

use derive_more::{
    Deref,
    DerefMut,
};
use itertools::Itertools;
use linked_hash_map::LinkedHashMap;
use linked_hash_set::LinkedHashSet;

use context::*;
use joined::*;
use partition::*;

use crate::{
    HashMap,
    index::indexer::Indexer,
    join::{
        context::node::context::NodeJoinContext,
        partition::{
            info::{
                JoinPartition,
                range::role::{
                    In,
                    Join,
                    Post,
                    Pre,
                },
                visit::{
                    PartitionBorders,
                    VisitPartition,
                },
            },
            splits::{
                HasPosSplits,
                offset::OffsetSplits,
                PosSplitRef,
            },
        },
    },
    split::{
        cache::{
            split::Split,
            SplitCache,
        },
        complete::position_splits,
    },
    traversal::{
        cache::key::SplitKey,
        folder::state::{
            FoldState,
            RootMode,
        },
        traversable::TraversableMut,
    },
    vertex::{
        child::Child,
        location::SubLocation,
        wide::Wide,
    },
};

pub mod context;
pub mod delta;
pub mod joined;
pub mod partition;

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
    fn extend<T: IntoIterator<Item=SplitKey>>(
        &mut self,
        iter: T,
    ) {
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

        let mut frontier = SplitFrontier::new(split_cache.leaves.iter().cloned().rev());
        let mut final_splits = HashMap::default();
        while let Some(key) = {
            frontier
                .pop_front()
                .and_then(|key| (key.index != root).then(|| key))
        } {
            if final_splits.get(&key).is_none() {
                let finals = {
                    let mut ctx = JoinContext::new(self.graph_mut(), &final_splits)
                        .node(key.index, &split_cache);
                    Self::join_node_partitions(&mut ctx)
                };

                for (key, split) in finals {
                    final_splits.insert(key, split);
                }
            }
            let top = split_cache
                .expect(&key)
                .top
                .iter()
                .sorted_by(|a, b| a.index.width().cmp(&b.index.width()))
                .cloned();
            frontier.extend(top);
        }
        let root_mode = split_cache.root_mode;
        let mut ctx = JoinContext::new(self.graph_mut(), &final_splits).node(root, &split_cache);
        Self::join_root_partitions(&mut ctx, root_mode)
    }
    pub fn index_partitions<'p, 't, S: HasPosSplits + 'p>(
        ctx: &mut NodeJoinContext<'p>,
        pos_splits: S,
    ) -> Vec<Child>
        where
            'p: 't,
    {
        let offset_splits = pos_splits.pos_splits();
        let len = offset_splits.len();
        assert!(len > 0);
        let mut iter = offset_splits.iter().map(|(&offset, splits)| OffsetSplits {
            offset,
            splits: splits.borrow().clone(),
        });
        let mut prev = iter.next().unwrap();
        let mut parts = Vec::with_capacity(1 + len);
        parts.push(Prefix::new(&prev).join_partition(ctx).into());
        for offset in iter {
            parts.push(Infix::new(&prev, &offset).join_partition(ctx).into());
            prev = offset;
        }
        parts.push(Postfix::new(prev).join_partition(ctx).into());
        println!("{:#?}", parts);
        parts
    }
    pub fn join_node_partitions(ctx: &mut NodeJoinContext) -> LinkedHashMap<SplitKey, Split> {
        let partitions = Self::index_partitions(ctx, ctx.pos_splits);
        assert_eq!(
            ctx.index.width(),
            partitions.iter().map(Child::width).sum::<usize>()
        );
        assert_eq!(partitions.len(), ctx.pos_splits.len() + 1,);
        ctx.merge_node(ctx.pos_splits, &partitions)
    }
    pub fn join_root_partitions(
        ctx: &mut NodeJoinContext,
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
                Self::join_prefix_root(ctx, offset, index)
            }
            RootMode::Postfix => {
                assert_eq!(num_offsets, 1);
                Self::join_postfix_root(ctx, offset, index)
            }
            RootMode::Infix => {
                assert_eq!(num_offsets, 2);
                let prev_offset = offset;
                let offset = offset_iter.next().unwrap();

                Self::join_infix_root(ctx, prev_offset, offset, index)
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
                if part.perfect.is_none() {
                    let post = Postfix::new(offset).join_partition(ctx).unwrap();
                    ctx.graph
                        .add_pattern_with_update(index, [part.index, post.index]);
                }
                part.index
            }
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
                if part.perfect.is_none() {
                    let pre = match Prefix::new(offset).join_partition(ctx) {
                        Ok(pre) => {
                            println!("{:#?}", pre);
                            pre.index
                        }
                        Err(c) => c,
                    };
                    ctx.graph.add_pattern_with_update(index, [pre, part.index]);
                }
                part.index
            }
            Err(c) => c,
        }
    }
    pub fn join_incomplete_infix<'p>(
        ctx: &mut NodeJoinContext<'p>,
        part: JoinedPartition<In<Join>>,
        loffset: PosSplitRef<'p>,
        roffset: PosSplitRef<'p>,
        index: Child,
    ) -> Child {
        let loffset = (loffset.0, loffset.1.clone());
        let roffset = (roffset.0, roffset.1.clone() - part.delta);

        if (None, None) == part.perfect.into() {
            // no perfect border
            //        [               ]
            // |     |      |      |     |   |
            let (offset, pre) = match Prefix::new(loffset).join_partition(ctx) {
                Ok(part) => ((roffset.0, roffset.1.clone() - part.delta), part.index),
                Err(ch) => (roffset, ch),
            };
            let post: Child = Postfix::new(offset).join_partition(ctx).into();
            ctx.graph
                .add_pattern_with_update(index, [pre, part.index, post]);
        } else if part.perfect.0 == part.perfect.1 {
            // perfect borders in same pattern
            //       [               ]
            // |     |       |       |      |
            let (ll, rl) = (part.perfect.0.unwrap(), part.perfect.1.unwrap());
            let lpos = loffset.1.pattern_splits[&ll].sub_index;
            let rpos = roffset.1.pattern_splits[&rl].sub_index;
            ctx.graph
                .replace_in_pattern(index.to_pattern_location(ll), lpos..rpos, [part.index])
        } else {
            // one or both are perfect in different patterns
            let loffset = (loffset.0, &loffset.1);
            let roffset = (roffset.0, &roffset.1);
            if let Some(rp) = part.perfect.1 {
                //           [              ]
                // |     |       |     |    |     |

                let (wrap_offset, li) = {
                    let pre_brds: PartitionBorders<Pre<Join>> =
                        Prefix::new(loffset).partition_borders(ctx);
                    let rp_brd = &pre_brds.borders[&rp];
                    let li = rp_brd.sub_index;
                    let lc = ctx
                        .graph
                        .expect_child_at(ctx.index.to_child_location(SubLocation::new(rp, li)));
                    let outer_offset =
                        NonZeroUsize::new(rp_brd.start_offset.unwrap().get() + lc.width()).unwrap();
                    (position_splits(ctx.patterns(), outer_offset), li)
                };
                let ri = roffset.1.pattern_splits[&rp].sub_index;

                //prev_offset.1 = prev_offset.1 - pre.delta;

                let wrap_patterns = Infix::new(&wrap_offset, roffset)
                    .info_partition(ctx)
                    .unwrap()
                    .to_joined_patterns(ctx);
                let wrap_pre = match Infix::new(wrap_offset, loffset).join_partition(ctx) {
                    Ok(p) => p.index,
                    Err(c) => c,
                };
                let wrapper = ctx.graph.insert_patterns(
                    std::iter::once(vec![wrap_pre, part.index]).chain(wrap_patterns.patterns),
                );
                let loc = index.to_pattern_location(rp);
                ctx.graph.replace_in_pattern(loc, li..ri, [wrapper]);

                //let patterns = wrap_patterns.patterns.clone();
                //offset.1 = offset.1 - wrap_patterns.delta;
                //let wrapper = ctx.graph.insert_patterns(
                //    std::iter::once(vec![pre.index, part.index])
                //        .chain(patterns),
                //);

                //let ri = offset.1.pattern_splits[&rp].sub_index;
                //let loc = index.to_pattern_location(rp);
                //ctx.graph.replace_in_pattern(
                //    loc,
                //    0..ri,
                //    [wrapper],
                //);
            }
            if let Some(lp) = part.perfect.0 {
                //       [                 ]
                // |     |       |      |      |   |

                // find wrapping offsets
                let (wrap_offset, ri) = {
                    let post_brds: PartitionBorders<Post<Join>> =
                        Postfix::new(roffset).partition_borders(ctx);
                    let lp_brd = &post_brds.borders[&lp];
                    let ri = lp_brd.sub_index;
                    let rc = ctx
                        .graph
                        .expect_child_at(ctx.index.to_child_location(SubLocation::new(lp, ri)));
                    let outer_offset =
                        NonZeroUsize::new(lp_brd.start_offset.unwrap().get() + rc.width()).unwrap();
                    (position_splits(ctx.patterns(), outer_offset), ri)
                };

                let li = loffset.1.pattern_splits[&lp].sub_index;

                let wrap_patterns = Infix::new(loffset, &wrap_offset)
                    .info_partition(ctx)
                    .unwrap()
                    .to_joined_patterns(ctx);
                let wrap_post = match Infix::new(roffset, wrap_offset).join_partition(ctx) {
                    Ok(p) => p.index,
                    Err(c) => c,
                };

                let wrapper = ctx.graph.insert_patterns(
                    std::iter::once(vec![part.index, wrap_post]).chain(wrap_patterns.patterns),
                );
                let loc = index.to_pattern_location(lp);
                ctx.graph.replace_in_pattern(loc, li..ri + 1, [wrapper]);
            }
        }
        part.index
    }
    pub fn join_infix_root<'p>(
        ctx: &mut NodeJoinContext<'p>,
        loffset: PosSplitRef<'p>,
        roffset: PosSplitRef<'p>,
        index: Child,
    ) -> Child {
        match Infix::new(loffset, roffset).join_partition(ctx) {
            Ok(part) => Self::join_incomplete_infix(ctx, part, loffset, roffset, index),
            Err(c) => c,
        }
    }
}
