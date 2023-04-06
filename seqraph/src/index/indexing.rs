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
        // todo: handle new/old offset positions
        let vert_cache = cache.entries.get_mut(&index.index).unwrap();
        let offsets = &mut vert_cache.positions;
        assert!(partitions.len() == offsets.len() + 1);
        let merges = self.merge_partitions(
            &partitions,
        );
        let len = offsets.len();
        for (i, (_, v)) in offsets.iter_mut().enumerate() {
            let lr = 0..i;
            let rr = i+1..len;
            let left = *merges.get(&lr).unwrap();
            let right = *merges.get(&rr).unwrap();
            if !lr.is_empty() || !lr.is_empty() {
                if let Some((&pid, _)) = v.pattern_splits.iter().find(|(_, s)| s.inner_offset.is_none()) {
                    self.graph_mut().replace_in_pattern(index.to_pattern_location(pid), 0.., [left, right]);
                } else {
                    self.graph_mut().add_pattern_with_update(index, [left, right]);
                }
            }
            v.final_split = Some(Split::new(left, right));
        }
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
            RootPartitions::Inner(pre, inner, post) => {
                graph.add_pattern_with_update(index, [&pre[..], &[*inner], &post[..]].concat());
            },
            RootPartitions::Prefix(inner, post) => {
                graph.add_pattern_with_update(index, [&[*inner], &post[..]].concat());
            },
            RootPartitions::Postfix(pre, inner) => {
                graph.add_pattern_with_update(index, [&pre[..], &[*inner]].concat());
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