use crate::*;

#[derive(Debug)]
pub struct JoinContext<'p> {
    pub index: Child,
    pub graph: RwLockWriteGuard<'p, Hypergraph>,
}
#[derive(Debug)]
pub struct Partitioner<'p> {
    pub ctx: JoinContext<'p>,
    pub cache: &'p SplitCache,
}
impl<'p> Deref for Partitioner<'p> {
    type Target = JoinContext<'p>;
    fn deref(&self) -> &Self::Target {
        &self.ctx
    }
}
impl<'p> DerefMut for Partitioner<'p> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.ctx
    }
}

impl<'p> JoinContext<'p> {
    pub fn patterns(
        &self,
    ) -> &ChildPatterns {
        self.graph.expect_child_patterns(self.index)
    }
}
impl<'p> Partitioner<'p> {
    /// calls `index_range` for each offset partition

    pub fn index_partition<'a, P: AsPartition<'a>>(
        &mut self,
        part: P,
    ) -> Result<JoinedPartition, Child> {
        part.join(self)
    }
    pub fn indexed_partition_patterns<'a, P: AsPartition<'a>>(
        &mut self,
        part: P,
    ) -> Result<JoinedPatterns, Child> {
        part.info_bundle(self)
            .map(|b| b.join_patterns(self))
    }
    pub fn index_partitions(
        &mut self,
        sub_splits: impl HasSubSplits,
    ) -> Vec<Child> {
        let offset_splits = sub_splits.sub_splits(self.cache);
        let len = offset_splits.len();
        assert!(len > 0);
        let mut iter = offset_splits.iter()
            .map(|(&offset, splits)|
                OffsetSplitsRef {
                    offset,
                    splits: splits.borrow(),
                });
        let mut prev = iter.next().unwrap();
        let mut parts = Vec::with_capacity(1 + len);
        parts.push(
            ((), prev).join(self).into()
        );
        for offset in iter {
            parts.push(
                (prev, offset).join(self).into()
            );
            prev = offset;
        }
        parts.push(
            (prev, ()).join(self).into()
        );
        parts
    }
}