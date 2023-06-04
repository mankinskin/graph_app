use crate::*;

pub mod joined;
pub use joined::*;
pub mod merge;
pub use merge::*;
pub mod delta;
pub use delta::*;


#[derive(Debug)]
pub struct JoinContext<'p> {
    pub graph: RwLockWriteGuard<'p, Hypergraph>,
    pub index: Child,
}
impl<'p> JoinContext<'p> {
    pub fn new(
        graph: RwLockWriteGuard<'p, Hypergraph>,
        index: Child,
    ) -> Self {
        Self {
            graph,
            index,
        }
    }
    pub fn patterns(
        &self,
    ) -> &ChildPatterns {
        self.graph.expect_child_patterns(self.index)
    }
}

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
        let splits = subgraph.into_split_graph(self);
        self.join_children(
            subgraph,
            splits,
        )
    }
    pub fn join_children(
        &mut self,
        subgraph: FoldState,
        mut splits: SplitCache,
    ) -> Child {
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
                let finals = JoinContext::new(
                    self.graph_mut(),
                    key.index,
                    //&mut splits,
                ).join_node();
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
        JoinContext::new(
            self.graph_mut(),
            subgraph.root,
            //&mut splits,
        ).join_root()
    }
}
pub trait JoinInfos {
    fn join_node_infos(
        &mut self,
    ) -> LinkedHashMap<SplitKey, Split> {
        let partitions = self.index_partitions(
            self.index,
        );
        assert_eq!(
            self.index.width(),
            partitions.iter().map(Child::width).sum::<usize>()
        );
        self.merge_node(
            &partitions,
        )
    }
    fn join_root_infos(
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
                match ((), offset).join_partition(self) {
                    Ok(part) => {
                        if let Some(pid) = part.perfect {
                            let pos = &offset.1.pattern_splits[&pid];
                            self.graph.replace_in_pattern(
                                index.to_pattern_location(pid),
                                0..pos.sub_index,
                                [part.index],
                            )
                        } else {
                            let post = (offset, ()).join_partition(self).unwrap();
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
                match (offset, ()).join_partition(self) {
                    Ok(part) => {
                        if let Some(pid) = part.perfect {
                            let pos = &offset.1.pattern_splits[&pid];
                            self.graph.replace_in_pattern(
                                index.to_pattern_location(pid),
                                pos.sub_index..,
                                [part.index],
                            )
                        } else {
                            let pre = ((), offset).join_partition(self).unwrap();
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

                match (prev_offset, offset).join_partition(self) {
                    Ok(part) => {
                        let mut prev_offset = (prev_offset.0, prev_offset.1.clone());
                        let mut offset = (offset.0, (offset.1.clone() - part.delta));

                        if (None, None) == part.perfect {
                            // no perfect border
                            //        [               ]
                            // |     |      |      |     |   |
                            let pre = ((), prev_offset).join_partition(self).unwrap();

                            let offset = (offset.0, &(offset.1.clone() - pre.delta));

                            let post = (offset, ()).join_partition(self).unwrap();
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
                                let pre = ((), &prev_offset).join_partition(self).unwrap();
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
                                let post = (offset, ()).join_partition(self).unwrap();

                                let li = prev_offset.1.pattern_splits[&lp].sub_index;
                                let mut info_bundle = (prev_offset, ())
                                    .info_bundle(self).unwrap();
                                // todo: skip lp in info_bundle already
                                info_bundle.bundle.remove(&lp);
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