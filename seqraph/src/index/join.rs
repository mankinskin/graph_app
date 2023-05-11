use crate::*;

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
        mut subgraph: FoldState,
    ) -> Child {
        let mut splits = subgraph.into_split_graph(self);
        // todo: how to get child splits of offsets induced by inner ranges?
        // - augment to split graph
        // or - locate dynamically (child is guaranteed to exist because inner range offset are always consistent)
        let mut frontier = SplitFrontier::new(splits.leaves.iter().cloned().rev());
        while let Some(key) = {
            frontier.pop_front()
                .and_then(|key|
                    (key.index != subgraph.root).then(|| key)
                )
        } {
            if splits.get_final_split(&key).is_none() {
                let finals = {
                    Partitioner {
                        ctx: JoinContext {
                            index: key.index,
                            graph: self.graph_mut(),
                        },
                        cache: &mut splits,
                    }.join_node()
                };
                for (key, split) in finals {
                    splits.expect_mut(&key).final_split = Some(split);
                }
            }
            //todo: store final split in frontier
            frontier.extend(
                splits.expect(&key).top.iter()
                    .sorted_by(|a, b| a.index.width().cmp(&b.index.width()))
                    .cloned()
            );
        }
        let mut partitioner = Partitioner {
            ctx: JoinContext {
                index: subgraph.root,
                graph: self.graph_mut(),
            },
            cache: &mut splits,
        };
        partitioner.join_root()
    }
}
impl<'p> Partitioner<'p> {
    pub fn join_node(
        &mut self,
    ) -> LinkedHashMap<SplitKey, Split> {
        let partitions = self.index_partitions(
            self.index,
        );
        self.merge_node(
            &partitions,
        )
    }
    pub fn join_root(
        &mut self,
    ) -> Child {
        let index = self.index;
        let root_mode = self.cache.root_mode;
        let offsets = index.sub_splits(self.cache);
        let num_offsets = offsets.len();
        let mut offset_iter = offsets.iter();
        let offset = offset_iter.next().unwrap();

        match root_mode {
            RootMode::Prefix => {
                assert_eq!(num_offsets, 1);
                match ((), offset).join(self) {
                    Ok(part) => {
                        if let Some(pid) = part.perfect.1 {
                            let pos = &offset.1.pattern_splits[&pid];
                            self.graph.replace_in_pattern(
                                index.to_pattern_location(pid),
                                0..pos.sub_index,
                                [part.index],
                            )
                        } else {
                            let post = (offset, ()).join(self).unwrap();
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
                match (offset, ()).join(self) {
                    Ok(part) => {
                        if let Some(pid) = part.perfect.1 {
                            let pos = &offset.1.pattern_splits[&pid];
                            self.graph.replace_in_pattern(
                                index.to_pattern_location(pid),
                                pos.sub_index..,
                                [part.index],
                            )
                        } else {
                            let pre = ((), offset).join(self).unwrap();
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

                match (prev_offset, offset).join(self) {
                    Ok(part) => {
                        let mut prev_offset = (prev_offset.0, prev_offset.1.clone());
                        let mut offset = (offset.0, (offset.1.clone() - part.delta));

                        if (None, None) == part.perfect {
                            // no perfect border
                            //        [               ]
                            // |     |      |      |     |   |
                            let pre = ((), prev_offset).join(self).unwrap();

                            let offset = (offset.0, &(offset.1.clone() - pre.delta));

                            let post = (offset, ()).join(self).unwrap();
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
                                let pre = ((), &prev_offset).join(self).unwrap();
                                prev_offset.1 = prev_offset.1 - pre.delta;

                                let wrap_patterns = ((), &offset)
                                    .info_bundle(self).unwrap()
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
                                let post = (offset, ()).join(self).unwrap();

                                let li = prev_offset.1.pattern_splits[&lp].sub_index;
                                let wrap_patterns = (prev_offset, ())
                                    .info_bundle(self).unwrap()
                                    .join_patterns(self);

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
    // merge 0, 1, 2, ... = [-0, 0-], [-1, 1-], 
    // merge [0, 1], [1, 2], ... = [[m0, 1-], [-0, m1]], [[m1, 2-], [-1, m2]]
    // merge [0, 1, 2], [1, 2, 3], ... = [[m01, 2-], [-0, m12]], [[m12, 3-], [-1, m23]]
    // merge [0, 1, 2, 3], [1, 2, 3, 4], ... = [[m012, 3-], [m01, m23], [-0, m123]], [[m123, 4-], [m12, m34], [-1, m234]]
    // merge [0, 1, 2, 3, 4], [1, 2, 3, 4, 5], ... = [
    //      [m0123, 4-],
    //      [m012, m34],
    //      [m01, m234],
    //      [-0, m1234]
    // ],
}