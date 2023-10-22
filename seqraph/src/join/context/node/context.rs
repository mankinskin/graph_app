use crate::*;

#[derive(Debug, Clone, Copy)]
pub struct NodeTraceContext<'p> {
    pub patterns: &'p ChildPatterns,
    pub index: Child,
}
impl<'p> NodeTraceContext<'p> {
    pub fn new(
        graph: &'p RwLockWriteGuard<'p, Hypergraph>,
        index: Child,
    ) -> Self {
        Self {
            patterns: graph.expect_child_patterns(index),
            index,
        }
    }
}

pub trait AsNodeTraceContext<'p>: 'p {
    fn as_trace_context<'t>(&'t self) -> NodeTraceContext<'t> where Self: 't, 'p: 't;
}
impl<'p> AsNodeTraceContext<'p> for NodeTraceContext<'p> {
    fn as_trace_context<'t>(&'t self) -> NodeTraceContext<'t> where Self: 't, 'p: 't {
        *self
    }
}
#[derive(Debug, Deref, DerefMut)]
pub struct NodeJoinContext<'p, S: HasSplitPos + 'p = SplitVertexCache> {
    #[deref]
    #[deref_mut]
    pub ctx: JoinContext<'p>,
    pub index: Child,
    pub split_pos: &'p SplitPos<S>,
}
impl<'p, S: HasSplitPos + 'p> AsNodeTraceContext<'p> for NodeJoinContext<'p, S> {
    fn as_trace_context<'t>(&'t self) -> NodeTraceContext<'t> where Self: 't, 'p: 't {
        NodeTraceContext {
            patterns: self.borrow().patterns(),
            index: self.borrow().index,
        }
    }
}

pub trait ToPatternContext<'p> {
    type PatternCtx<'a>: AsPatternTraceContext<'p> where Self: 'a, 'a: 'p;
    fn to_pattern_context<'t>(self,  pattern_id: &PatternId) -> Self::PatternCtx<'t> where Self: 't, 't: 'p;
}
impl<'p, SP: HasSplitPos + 'p> AsPatternContext<'p> for NodeJoinContext<'p, SP> {
    type PatternCtx<'a> = PatternJoinContext<'a> where Self: 'a, 'a: 'p;
    fn as_pattern_context<'t>(&'t self, pattern_id: &PatternId) -> Self::PatternCtx<'t> where Self: 't, 't: 'p {

        let ctx = PatternTraceContext {
            loc: self.index.to_pattern_location(*pattern_id),
            pattern: self.as_trace_context().patterns.get(pattern_id).unwrap(),
        };
        PatternJoinContext {
            //graph: self.graph,
            ctx,
            sub_splits: self.borrow().sub_splits,
        }
    }
}
pub trait AsPatternContext<'p> {
    type PatternCtx<'a>: AsPatternTraceContext<'p> where Self: 'a, 'a: 'p;
    fn as_pattern_context<'t>(&'t self,  pattern_id: &PatternId) -> Self::PatternCtx<'t> where Self: 't, 't: 'p;
}
impl<'p> AsPatternContext<'p> for NodeTraceContext<'p> {
    type PatternCtx<'a> = PatternTraceContext<'p> where Self: 'a, 'a: 'p;
    fn as_pattern_context<'t>(&'t self, pattern_id: &PatternId) -> Self::PatternCtx<'t> where Self: 't, 't: 'p {
        PatternTraceContext {
            loc: self.index.to_pattern_location(*pattern_id),
            pattern: self.as_trace_context().patterns.get(pattern_id).unwrap(),
        }
    }
}
impl<'p, SP: HasSplitPos + 'p> NodeJoinContext<'p, SP> {
    pub fn new(
        ctx: JoinContext<'p>,
        index: Child,
        split_pos: &'p SP,
    ) -> Self {
        Self {
            ctx,
            index,
            split_pos: split_pos.split_pos(),
        }
    }
    pub fn patterns(&self) -> &ChildPatterns {
        self.ctx.graph.expect_child_patterns(self.index)
    }
}
impl<'p> NodeJoinContext<'p> {
    pub fn index_partitions<'t, S: HasSplitPos + 'p>(
        &mut self,
        split_pos: S,
    ) -> Vec<Child>
        where 'p: 't
    {
        let offset_splits = split_pos.split_pos();
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
            Prefix::new(&prev).join_partition(self).into()
        );
        for offset in iter {
            parts.push(
                Infix::new(&prev, &offset).join_partition(self).into()
            );
            prev = offset;
        }
        parts.push(
            Postfix::new(prev).join_partition(self).into()
        );
        println!("{:#?}", parts);
        parts
    }
    pub fn join_node_partitions(
        &mut self,
    ) -> LinkedHashMap<SplitKey, Split> {
        let partitions = self.index_partitions(
            self.split_pos,
        );
        assert_eq!(
            self.index.width(),
            partitions.iter().map(Child::width).sum::<usize>()
        );
        assert_eq!(
            partitions.len(),
            self.split_pos.len() + 1,
        );
        self.merge_node(
            self.split_pos,
            &partitions,
        )
    }
    pub fn join_root_partitions(
        &mut self,
        root_mode: RootMode,
    ) -> Child {
        let index = self.index;
        let offsets = self.split_pos;
        let num_offsets = offsets.len();
        let mut offset_iter = offsets.iter();
        let offset = offset_iter.next().unwrap();

        match root_mode {
            RootMode::Prefix => {
                assert_eq!(num_offsets, 1);
                match Prefix::new(offset).join_partition(self) {
                    Ok(part) => {
                        if let Some(pid) = part.perfect {
                            let pos = &offset.1.pattern_splits[&pid];
                            self.graph.replace_in_pattern(
                                index.to_pattern_location(pid),
                                0..pos.sub_index,
                                [part.index],
                            )
                        } else {
                            let post = Postfix::new(offset).join_partition(self).unwrap();
                            self.graph.add_pattern_with_update(
                                index,
                                [part.index, post.index],
                            );
                        }
                        part.index
                    },
                    Err(c) => c,
                }
            },
            RootMode::Postfix => {
                assert_eq!(num_offsets, 1);
                match Postfix::new(offset).join_partition(self) {
                    Ok(part) => {
                        println!("{:#?}", part);
                        if let Some(pid) = part.perfect {
                            let pos = &offset.1.pattern_splits[&pid];
                            self.graph.replace_in_pattern(
                                index.to_pattern_location(pid),
                                pos.sub_index..,
                                [part.index],
                            )
                        } else {
                            let pre = Prefix::new(offset).join_partition(self).unwrap();
                            println!("{:#?}", pre);
                            self.graph.add_pattern_with_update(index,
                                [pre.index, part.index],
                            );
                        }
                        part.index
                    },
                    Err(c) => c,
                }
            },
            RootMode::Infix => {
                assert_eq!(num_offsets, 2);
                let prev_offset = offset;
                let offset = offset_iter.next().unwrap();

                match Infix::new(prev_offset, offset).join_partition(self) {
                    Ok(part) => {
                        let mut prev_offset = (prev_offset.0, prev_offset.1.clone());
                        let mut offset = (offset.0, offset.1.clone() - part.delta);

                        if (None, None) == part.perfect {
                            // no perfect border
                            //        [               ]
                            // |     |      |      |     |   |
                            let pre = Prefix::new(prev_offset).join_partition(self).unwrap();

                            let offset = (offset.0, &(offset.1.clone() - pre.delta));

                            let post = Postfix::new(offset).join_partition(self).unwrap();
                            self.graph.add_pattern_with_update(
                                index,
                                [pre.index, part.index, post.index],
                            );
                        } else if part.perfect.0 == part.perfect.1 {
                            // perfect borders in same pattern
                            //       [               ]
                            // |     |       |       |      |
                            let (ll, rl) = (part.perfect.0.unwrap(), part.perfect.1.unwrap());
                            let lpos = prev_offset.1.pattern_splits[&ll].sub_index;
                            let rpos = offset.1.pattern_splits[&rl].sub_index;
                            self.graph.replace_in_pattern(
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
                                let pre = Prefix::new(prev_offset.clone()).join_partition(self).unwrap();
                                prev_offset.1 = prev_offset.1 - pre.delta;

                                let wrap_patterns = Prefix::new(offset.clone())
                                    .info_partition(self).unwrap()
                                    .join_patterns(self);
                                let patterns = wrap_patterns.patterns().clone();
                                offset.1 = offset.1 - wrap_patterns.delta;
                                let wrapper = self.graph.insert_patterns(
                                    std::iter::once(vec![pre.index, part.index])
                                        .chain(patterns),
                                );

                                let ri = offset.1.pattern_splits[&rp].sub_index;
                                let loc = index.to_pattern_location(rp);
                                self.graph.replace_in_pattern(
                                    loc,
                                    0..ri,
                                    [wrapper],
                                );
                            }
                            if let Some(lp) = part.perfect.0 {
                                //       [                 ]
                                // |     |       |      |      |
                                let post = Postfix::new(offset).join_partition(self).unwrap();

                                let li = prev_offset.1.pattern_splits[&lp].sub_index;
                                let mut info_bundle = Postfix::new(prev_offset)
                                    .info_partition(self).unwrap();
                                // todo: skip lp in info_bundle already
                                info_bundle.patterns.remove(&lp);
                                let wrap_patterns = info_bundle.join_patterns(self);

                                let wrapper = self.graph.insert_patterns(
                                    std::iter::once(vec![part.index, post.index])
                                        .chain(wrap_patterns.patterns()),
                                );
                                let loc = index.to_pattern_location(lp);
                                self.graph.replace_in_pattern(
                                    loc,
                                    li..,
                                    [wrapper],
                                );
                            }
                        }
                        part.index
                    },
                    Err(c) => c,
                }
            }
        }
    }
}