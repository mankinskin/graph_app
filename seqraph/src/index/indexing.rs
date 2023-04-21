use crate::*;

#[derive(Debug, Default, Deref, DerefMut)]
pub struct SplitFrontier {
    pub queue: LinkedHashSet<SplitKey>,
}
impl SplitFrontier {
    pub fn new(leaves: impl IntoIterator<Item=SplitKey>) -> Self {
        Self {
            queue: LinkedHashSet::from_iter(leaves),
        }
    }
}
impl Extend<SplitKey> for SplitFrontier {
    fn extend<T: IntoIterator<Item = SplitKey>>(&mut self, iter: T) {
        self.queue.extend(iter)
    }
}
impl Indexer {
    pub fn index_subgraph(
        &mut self,
        mut subgraph: FoldState,
    ) -> Child {
        let mut splits = subgraph.into_split_graph(self);
        let mut frontier = SplitFrontier::new(splits.leaves.iter().cloned().rev());
        while let Some(key) = frontier.pop_front() {
            if key.index == subgraph.root {
                // todo handle roots
                return self.join_root(
                    &mut splits,
                    &key.index,
                );
            } else {
                if splits.get_final_split(&key).is_none() {
                    self.join_node(
                        &mut splits,
                        &key.index,
                    )
                }
                frontier.extend(
                    splits.expect(&key).top.iter()
                        .sorted_by(|a, b| a.index.width().cmp(&b.index.width()))
                        .cloned()
                );
            }
        }
        unreachable!();
    }

    pub fn join_node(
        &mut self,
        cache: &mut SplitCache,
        index: &Child,
    ) {
        let partitions = self.partition_node(
            cache,
            index,
        );
        self.merge_node(
            index,
            &partitions,
            cache,
        )
    }
    pub fn join_root(
        &mut self,
        cache: &mut SplitCache,
        index: &Child,
    ) -> Child {
        let partitions = self.partition_root(
            cache,
            index,
        );
        let mut graph = self.graph_mut();
        match &partitions {
            RootPartitions::Infix(pre, first_offset, inner, second_offset, post) => {
                match (first_offset, second_offset) {
                    (Some(ll), Some(rl)) =>
                        if ll.pattern_id == rl.pattern_id {
                            graph.replace_in_pattern(
                                index.to_pattern_location(ll.pattern_id),
                                ll.sub_index..rl.sub_index,
                                [*inner],
                            )
                        } else {

                        },
                    (None, Some(rl)) => {
                        let loc = index.to_pattern_location(rl.pattern_id);
                        let pattern = graph.expect_pattern_at(loc)[0..rl.sub_index].to_vec();
                        let wrapper = graph.insert_patterns([
                            pattern,
                            [pre.borrow(), &[*inner][..]].concat(),
                        ]);
                        graph.replace_in_pattern(
                            loc,
                            0..rl.sub_index,
                            [wrapper],
                        )
                    },
                    (Some(ll), None) => {
                        let loc = index.to_pattern_location(ll.pattern_id);
                        let pattern = graph.expect_pattern_at(loc)[ll.sub_index..].to_vec();
                        let wrapper = graph.insert_patterns([
                            pattern,
                            [&[*inner][..], post.borrow()].concat(),
                        ]);
                        graph.replace_in_pattern(
                            loc,
                            ll.sub_index..,
                            [wrapper],
                        )
                    },
                    (None, None) => {
                        graph.add_pattern_with_update(
                            index,
                            [&pre[..], &[*inner], &post[..]].concat(),
                        );
                    },
                }
            },
            RootPartitions::Prefix(inner, offset, post) => {
                if let Some(loc) = offset {
                    graph.replace_in_pattern(
                        index.to_pattern_location(loc.pattern_id),
                        loc.sub_index..,
                        [*inner],
                    )
                } else {
                    graph.add_pattern_with_update(index, [&[*inner], &post[..]].concat());
                }
            },
            RootPartitions::Postfix(pre, offset, inner) => {
                if let Some(loc) = offset {
                    graph.replace_in_pattern(
                        index.to_pattern_location(loc.pattern_id),
                        0..loc.sub_index,
                        [*inner],
                    )
                } else {
                    graph.add_pattern_with_update(index, [&pre[..], &[*inner]].concat());
                }
            },
        }
        *partitions.inner()
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