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
    graph::vertex::{
        child::Child, has_vertex_index::HasVertexIndex, location::SubLocation, pattern::id::PatternId, wide::Wide, ChildPatterns
    },
    join::{
        context::{
            node::merge::NodeMergeContext, pattern::PatternJoinContext, JoinContext
        }, joined::partition::JoinedPartition, partition::{
            join::JoinPartition, Join
        }
    },
    partition::{
        context::{
            AsNodeTraceContext,
            NodeTraceContext,
        }, info::{range::role::{
                In,
                Post,
                Pre,
            }, InfoPartition, PartitionBorders}, pattern::{
            GetPatternContext, GetPatternTraceContext, PatternTraceContext
        }, splits::{
            has_splits::HasPosSplits, offset::OffsetSplit, pos::PosSplitContext, PosSplitOf,
        }, Infix, Postfix, Prefix
    },
    split::{
        cache::{
            position_splits,
            split::Split,
            vertex::SplitVertexCache,
        },
        VertexSplitPos,
    },
    traversal::cache::{
        entry::RootMode,
        key::SplitKey,
    }, HashMap,
};

#[derive(Debug, Deref, DerefMut)]
pub struct NodeJoinContext<'a: 'b, 'b> {
    #[deref]
    #[deref_mut]
    pub ctx: &'b mut JoinContext<'a>,
    pub index: Child,
    //pub vertex_cache: SplitVertexCache,
    pub finished_splits: &'b HashMap<SplitKey, Split>,
}

impl<'a: 'b, 'b> AsNodeTraceContext for NodeJoinContext<'a, 'b> {
    fn as_trace_context<'t>(&'t self) -> NodeTraceContext<'t>
    where
        Self: 't,
        'a: 't,
    {
        NodeTraceContext {
            patterns: self.patterns(),
            index: self.borrow().index,
        }
    }
}
impl<'a: 'b, 'b> GetPatternTraceContext for NodeJoinContext<'a, 'b> {
    fn get_pattern_trace_context<'c>(
        &'c self,
        pattern_id: &PatternId,
    ) -> PatternTraceContext<'c>
    where
        Self: 'c,
    {
        PatternTraceContext {
            loc: self.index.to_pattern_location(*pattern_id),
            pattern: self.as_trace_context().patterns.get(pattern_id).unwrap(),
        }
    }
}
impl<'a: 'b, 'b> GetPatternContext for NodeJoinContext<'a, 'b> {
    type PatternCtx<'c>
        = PatternJoinContext<'c>
    where
        Self: 'c;

    fn get_pattern_context<'c>(
        &'c self,
        pattern_id: &PatternId,
    ) -> Self::PatternCtx<'c>
        where Self: 'c,
    {
        let ctx = self.get_pattern_trace_context(pattern_id);
        //let pos_splits = self.vertex_cache().pos_splits();
        PatternJoinContext {
            ctx,
            sub_splits: &self.finished_splits
            //pos_splits
            //    .iter()
            //    .map(|pos| PosSplitContext::from(pos).fetch_split(&self.ctx.split_cache))
            //    .collect(),
        }
    }
}
impl<'a: 'b, 'b> NodeJoinContext<'a, 'b> {
    pub fn patterns(&self) -> &ChildPatterns
    {
        self.ctx.graph.expect_child_patterns(self.index)
    }
}

impl<'a: 'b, 'b> NodeJoinContext<'a, 'b> {
    pub fn vertex_cache<'c>(&'c self) -> &'c SplitVertexCache {
        self.split_cache.entries.get(&self.index.vertex_index()).unwrap()
    }
    pub fn join_partitions(&mut self) -> LinkedHashMap<SplitKey, Split>
    {
        let partitions = self.index_partitions();
        assert_eq!(
            self.index.width(),
            partitions.iter().map(Child::width).sum::<usize>()
        );
        let pos_splits = self.vertex_cache().pos_splits();
        assert_eq!(partitions.len(), pos_splits.len() + 1,);
        NodeMergeContext::new(self).merge_node(&partitions)
    }
    pub fn index_partitions(&mut self) -> Vec<Child>
    {
        let offset_splits = self.vertex_cache().pos_splits().clone();
        let len = offset_splits.len();
        assert!(len > 0);
        let mut iter = offset_splits.iter().map(|(&offset, splits)| OffsetSplit {
            offset,
            splits: (splits.borrow() as &VertexSplitPos).clone(),
        });

        let mut prev = iter.next().unwrap();
        let mut parts = Vec::with_capacity(1 + len);
        parts.push(Prefix::new(&prev).join_partition(self).into());
        for offset in iter {
            parts.push(Infix::new(&prev, &offset).join_partition(self).into());
            prev = offset;
        }
        parts.push(Postfix::new(prev).join_partition(self).into());
        println!("{:#?}", parts);
        parts
    }
    pub fn join_root_partitions(
        &mut self,
    ) -> Child
    {
        let root_mode = self.split_cache.root_mode;
        let index = self.index;
        let offsets = self.vertex_cache().pos_splits().clone();
        let mut offset_iter = offsets.iter().map(PosSplitContext::from);
        let offset = offset_iter.next().unwrap();

        let x = match root_mode {
            RootMode::Prefix => Prefix::new(offset)
                .join_partition(self)
                .inspect(|part| {
                    if part.perfect.is_none() {
                        let post = Postfix::new(offset).join_partition(self).unwrap();
                        self.graph
                            .add_pattern_with_update(index, [part.index, post.index]);
                    }
                })
                .map(|part| part.index),
            RootMode::Postfix => Postfix::new(offset)
                .join_partition(self)
                .inspect(|part| {
                    if part.perfect.is_none() {
                        let pre = match Prefix::new(offset).join_partition(self) {
                            Ok(pre) => {
                                println!("{:#?}", pre);
                                pre.index
                            }
                            Err(c) => c,
                        };
                        self.graph.add_pattern_with_update(index, [pre, part.index]);
                    }
                })
                .map(|part| part.index),
            RootMode::Infix => {
                let loffset = offset;
                let roffset = offset_iter.next().unwrap();
                Infix::new(loffset, roffset)
                    .join_partition(self)
                    .map(|part| self.join_incomplete_infix(
                        part,
                        loffset,
                        roffset,
                        index,
                    ))
            }
        }
        .unwrap_or_else(|c| c);
        x
    }

    pub fn join_incomplete_infix<'c>(
        &mut self,
        part: JoinedPartition<In<Join>>,
        loffset: PosSplitContext<'c, PosSplitOf<SplitVertexCache>>,
        roffset: PosSplitContext<'c, PosSplitOf<SplitVertexCache>>,
        index: Child,
    ) -> Child {
        let loffset = (*loffset.pos, loffset.split.clone());
        let roffset = (*roffset.pos, roffset.split.clone() - part.delta);

        if (None, None) == part.perfect.into() {
            // no perfect border
            //        [               ]
            // |     |      |      |     |   |
            let (offset, pre) = match Prefix::new(loffset).join_partition(self) {
                Ok(part) => ((roffset.0, (roffset.1.clone() - part.delta)), part.index),
                Err(ch) => (roffset, ch),
            };
            let post: Child = Postfix::new(offset).join_partition(self).into();
            self.graph
                .add_pattern_with_update(index, [pre, part.index, post]);
        } else if part.perfect.0 == part.perfect.1 {
            // perfect borders in same pattern
            //       [               ]
            // |     |       |       |      |
            let (ll, rl) = (part.perfect.0.unwrap(), part.perfect.1.unwrap());
            let lpos = loffset.1.pattern_splits[&ll].sub_index;
            let rpos = roffset.1.pattern_splits[&rl].sub_index;
            self.graph
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
                        Prefix::new(loffset).partition_borders(self);
                    let rp_brd = &pre_brds.borders[&rp];
                    let li = rp_brd.sub_index;
                    let lc = self
                        .graph
                        .expect_child_at(self.index.to_child_location(SubLocation::new(rp, li)));
                    let outer_offset =
                        NonZeroUsize::new(rp_brd.start_offset.unwrap().get() + lc.width()).unwrap();
                    (position_splits(self.patterns(), outer_offset), li)
                };
                let ri = roffset.1.pattern_splits[&rp].sub_index;

                //prev_offset.1 = prev_offset.1 - pre.delta;

                let wrap_patterns = Infix::new(&wrap_offset, roffset)
                    .info_partition(self)
                    .unwrap()
                    .to_joined_patterns(self);
                let wrap_pre = match Infix::new(wrap_offset, loffset).join_partition(self) {
                    Ok(p) => p.index,
                    Err(c) => c,
                };
                let wrapper = self.graph.insert_patterns(
                    std::iter::once(vec![wrap_pre, part.index]).chain(wrap_patterns.patterns),
                );
                let loc = index.to_pattern_location(rp);
                self.graph.replace_in_pattern(loc, li..ri, [wrapper]);

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
                    let rc = self
                        .graph
                        .expect_child_at(self.index.to_child_location(SubLocation::new(lp, ri)));
                    let outer_offset =
                        NonZeroUsize::new(lp_brd.start_offset.unwrap().get() + rc.width()).unwrap();
                    (position_splits(self.patterns(), outer_offset), ri)
                };

                let li = loffset.1.pattern_splits[&lp].sub_index;

                let wrap_patterns = Infix::new(loffset, &wrap_offset)
                    .info_partition(self)
                    .unwrap()
                    .to_joined_patterns(self);
                let wrap_post = match Infix::new(roffset, wrap_offset).join_partition(self) {
                    Ok(p) => p.index,
                    Err(c) => c,
                };

                let wrapper = self.graph.insert_patterns(
                    std::iter::once(vec![part.index, wrap_post]).chain(wrap_patterns.patterns),
                );
                let loc = index.to_pattern_location(lp);
                self.graph.replace_in_pattern(loc, li..ri + 1, [wrapper]);
            }
        }
        part.index
    }
}
