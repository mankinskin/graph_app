use std::{
    borrow::Borrow,
    num::NonZeroUsize,
};

use derive_more::{
    Deref,
    DerefMut,
};
use linked_hash_map::LinkedHashMap;

use crate::{
    interval::{
        IntervalGraph,
        partition::{
            Infix,
            Postfix,
            Prefix,
            info::{
                InfoPartition,
                borders::PartitionBorders,
                range::role::{
                    In,
                    Post,
                    Pre,
                },
            },
        },
    },
    join::{
        context::{
            frontier::FrontierSplitIterator,
            node::merge::NodeMergeCtx,
            pattern::PatternJoinCtx,
        },
        joined::partition::JoinedPartition,
        partition::{
            Join,
            JoinPartition,
            info::JoinPartitionInfo,
        },
    },
    split::{
        Split,
        SplitMap,
        cache::{
            position::PosKey,
            vertex::SplitVertexCache,
        },
        position_splits,
        vertex::{
            ChildTracePositions,
            PosSplitCtx,
            VertexSplits,
            output::RootMode,
        },
    },
};
use context_trace::{
    graph::{
        HypergraphRef,
        vertex::{
            ChildPatterns,
            child::Child,
            has_vertex_index::HasVertexIndex,
            location::SubLocation,
            pattern::id::PatternId,
            wide::Wide,
        },
    },
    trace::{
        has_graph::HasGraphMut,
        node::{
            AsNodeTraceCtx,
            NodeTraceCtx,
        },
        pattern::{
            GetPatternCtx,
            GetPatternTraceCtx,
            PatternTraceCtx,
        },
    },
};

#[derive(Debug)]
pub struct LockedFrontierCtx<'a> {
    pub trav: <HypergraphRef as HasGraphMut>::GuardMut<'a>,
    pub interval: &'a IntervalGraph,
    pub splits: &'a SplitMap,
}
impl<'a> LockedFrontierCtx<'a> {
    pub fn new(ctx: &'a mut FrontierSplitIterator) -> Self {
        Self {
            trav: ctx.trav.graph_mut(),
            interval: &ctx.frontier.interval,
            splits: &ctx.splits,
        }
    }
}
#[derive(Debug, Deref, DerefMut)]
pub struct NodeJoinCtx<'a> {
    #[deref]
    #[deref_mut]
    pub ctx: LockedFrontierCtx<'a>,
    pub index: Child,
}

impl<'a> NodeJoinCtx<'a> {
    pub fn new(
        index: Child,
        ctx: &'a mut FrontierSplitIterator,
    ) -> Self {
        NodeJoinCtx {
            index,
            ctx: LockedFrontierCtx::new(ctx),
        }
    }
}
impl<'a: 'b, 'b> AsNodeTraceCtx for NodeJoinCtx<'a> {
    fn as_trace_context<'t>(&'t self) -> NodeTraceCtx<'t>
    where
        Self: 't,
        'a: 't,
    {
        NodeTraceCtx {
            patterns: self.patterns(),
            index: self.borrow().index,
        }
    }
}
impl<'a: 'b, 'b> GetPatternTraceCtx for NodeJoinCtx<'a> {
    fn get_pattern_trace_context<'c>(
        &'c self,
        pattern_id: &PatternId,
    ) -> PatternTraceCtx<'c>
    where
        Self: 'c,
    {
        PatternTraceCtx {
            loc: self.index.to_pattern_location(*pattern_id),
            pattern: self.as_trace_context().patterns.get(pattern_id).unwrap(),
        }
    }
}
impl<'a: 'b, 'b> GetPatternCtx for NodeJoinCtx<'a> {
    type PatternCtx<'c>
        = PatternJoinCtx<'c>
    where
        Self: 'c;

    fn get_pattern_context<'c>(
        &'c self,
        pattern_id: &PatternId,
    ) -> Self::PatternCtx<'c>
    where
        Self: 'c,
    {
        let ctx = self.get_pattern_trace_context(pattern_id);
        //let pos_splits = self.vertex_cache().pos_splits();
        PatternJoinCtx {
            ctx,
            splits: &self.splits, //pos_splits
                                  //    .iter()
                                  //    .map(|pos| PosSplitCtx::from(pos).fetch_split(&self.ctx.interval))
                                  //    .collect(),
        }
    }
}
impl<'a: 'b, 'b> NodeJoinCtx<'a> {
    pub fn patterns(&self) -> &ChildPatterns {
        self.ctx.trav.expect_child_patterns(self.index)
    }
}

impl<'a: 'b, 'b> NodeJoinCtx<'a> {
    pub fn vertex_cache<'c>(&'c self) -> &'c SplitVertexCache {
        self.interval.cache.get(&self.index.vertex_index()).unwrap()
    }
    pub fn join_partitions(&mut self) -> LinkedHashMap<PosKey, Split> {
        let partitions = self.index_partitions();
        assert_eq!(
            self.index.width(),
            partitions.iter().map(Child::width).sum::<usize>()
        );
        let pos_splits = self.vertex_cache();
        assert_eq!(partitions.len(), pos_splits.len() + 1,);
        NodeMergeCtx::new(self).merge_node(&partitions)
    }
    pub fn index_partitions(&mut self) -> Vec<Child> {
        let pos_splits = self.vertex_cache().clone();
        let len = pos_splits.len();
        assert!(len > 0);
        let mut iter = pos_splits.iter().map(|(&pos, splits)| VertexSplits {
            pos,
            splits: (splits.borrow() as &ChildTracePositions).clone(),
        });

        let mut prev = iter.next().unwrap();
        let mut parts = Vec::with_capacity(1 + len);
        parts.push(Prefix::new(&prev).join_partition(self).into());
        for offset in iter {
            parts.push(Infix::new(&prev, &offset).join_partition(self).into());
            prev = offset;
        }
        parts.push(Postfix::new(prev).join_partition(self).into());
        //println!("{:#?}", parts);
        parts
    }
    pub fn join_root_partitions(&mut self) -> Child {
        let root_mode = self.interval.cache.root_mode;
        let index = self.index;
        let offsets = self.vertex_cache().clone();
        let mut offset_iter = offsets.iter().map(PosSplitCtx::from);
        let offset = offset_iter.next().unwrap();

        let x = match root_mode {
            RootMode::Prefix => Prefix::new(offset)
                .join_partition(self)
                .inspect(|part| {
                    if part.perfect.is_none() {
                        let post =
                            Postfix::new(offset).join_partition(self).unwrap();
                        self.ctx.trav.add_pattern_with_update(index, vec![
                            part.index, post.index,
                        ]);
                    }
                })
                .map(|part| part.index),
            RootMode::Postfix => Postfix::new(offset)
                .join_partition(self)
                .inspect(|part| {
                    if part.perfect.is_none() {
                        let pre = match Prefix::new(offset).join_partition(self)
                        {
                            Ok(pre) => {
                                //println!("{:#?}", pre);
                                pre.index
                            },
                            Err(c) => c,
                        };
                        self.ctx.trav.add_pattern_with_update(index, vec![
                            pre, part.index,
                        ]);
                    }
                })
                .map(|part| part.index),
            RootMode::Infix => {
                let loffset = offset;
                let roffset = offset_iter.next().unwrap();
                Infix::new(loffset, roffset)
                    .join_partition(self)
                    .map(|part| {
                        self.join_incomplete_infix(
                            part, loffset, roffset, index,
                        )
                    })
            },
        }
        .unwrap_or_else(|c| c);
        x
    }

    pub fn join_incomplete_infix<'c>(
        &mut self,
        part: JoinedPartition<In<Join>>,
        loffset: PosSplitCtx<'c>,
        roffset: PosSplitCtx<'c>,
        index: Child,
    ) -> Child {
        let loffset = (*loffset.pos, loffset.split.clone());
        let roffset = (*roffset.pos, roffset.split.clone() - part.delta);

        if (None, None) == part.perfect.into() {
            // no perfect border
            //        [               ]
            // |     |      |      |     |   |
            let (offset, pre) = match Prefix::new(loffset).join_partition(self)
            {
                Ok(part) =>
                    ((roffset.0, (roffset.1.clone() - part.delta)), part.index),
                Err(ch) => (roffset, ch),
            };
            let post: Child = Postfix::new(offset).join_partition(self).into();
            self.trav
                .add_pattern_with_update(index, vec![pre, part.index, post]);
        } else if part.perfect.0 == part.perfect.1 {
            // perfect borders in same pattern
            //       [               ]
            // |     |       |       |      |
            let (ll, rl) = (part.perfect.0.unwrap(), part.perfect.1.unwrap());
            let lpos = loffset.1.pattern_splits[&ll].sub_index;
            let rpos = roffset.1.pattern_splits[&rl].sub_index;
            self.ctx.trav.replace_in_pattern(
                index.to_pattern_location(ll),
                lpos..rpos,
                vec![part.index],
            )
        } else {
            // one or both are perfect in different patterns
            let loffset = (loffset.0, &loffset.1);
            let roffset = (roffset.0, &roffset.1);
            if let Some(rp) = part.perfect.1 {
                //           [              ]
                // |     |       |     |    |     |

                let (wrap_offset, li) = {
                    let pre_brds: PartitionBorders<Pre<Join>> =
                        Prefix::new(loffset).partition_borders(self);
                    let rp_brd = &pre_brds.borders[&rp];
                    let li = rp_brd.sub_index;
                    let lc = self.trav.expect_child_at(
                        self.index.to_child_location(SubLocation::new(rp, li)),
                    );
                    let outer_offset = NonZeroUsize::new(
                        rp_brd.start_offset.unwrap().get() + lc.width(),
                    )
                    .unwrap();
                    (position_splits(self.patterns(), outer_offset), li)
                };
                let ri = roffset.1.pattern_splits[&rp].sub_index;

                //prev_offset.1 = prev_offset.1 - pre.delta;

                let info = Infix::new(&wrap_offset, roffset)
                    .info_partition(self)
                    .unwrap();
                let wrap_patterns =
                    JoinPartitionInfo::from(info).to_joined_patterns(self);
                let wrap_pre = match Infix::new(wrap_offset, loffset)
                    .join_partition(self)
                {
                    Ok(p) => p.index,
                    Err(c) => c,
                };
                let wrapper = self.trav.insert_patterns(
                    std::iter::once(vec![wrap_pre, part.index])
                        .chain(wrap_patterns.patterns),
                );
                let loc = index.to_pattern_location(rp);
                self.trav.replace_in_pattern(loc, li..ri, vec![wrapper]);

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
                        Postfix::new(roffset).partition_borders(self);
                    let lp_brd = &post_brds.borders[&lp];
                    let ri = lp_brd.sub_index;
                    let rc = self.trav.expect_child_at(
                        self.index.to_child_location(SubLocation::new(lp, ri)),
                    );
                    let outer_offset = NonZeroUsize::new(
                        lp_brd.start_offset.unwrap().get() + rc.width(),
                    )
                    .unwrap();
                    (position_splits(self.patterns(), outer_offset), ri)
                };

                let li = loffset.1.pattern_splits[&lp].sub_index;

                let info = Infix::new(loffset, &wrap_offset)
                    .info_partition(self)
                    .unwrap();
                let wrap_patterns =
                    JoinPartitionInfo::from(info).to_joined_patterns(self);
                let wrap_post = match Infix::new(roffset, wrap_offset)
                    .join_partition(self)
                {
                    Ok(p) => p.index,
                    Err(c) => c,
                };

                let wrapper = self.trav.insert_patterns(
                    std::iter::once(vec![part.index, wrap_post])
                        .chain(wrap_patterns.patterns),
                );
                let loc = index.to_pattern_location(lp);
                self.trav.replace_in_pattern(loc, li..ri + 1, vec![wrapper]);
            }
        }
        part.index
    }
}
