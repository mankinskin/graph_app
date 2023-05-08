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
    ) -> IndexedPartition {
        part.as_partition()
            .join(self)
    }
    pub fn indexed_partition_patterns<'a, P: AsPartition<'a>>(
        &mut self,
        part: P,
    ) -> Result<IndexedPatterns, IndexedPartition> {
        part.as_partition()
            .info_bundle(self)
            .map(|b| b.join_patterns(self))
    }
    pub fn index_partitions(
        &mut self,
        sub_splits: impl HasSubSplits,
    ) -> Vec<IndexedPartition> {
        let offset_splits = sub_splits.sub_splits();
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
            FirstPartition {
                inner: prev,
            }.join(self)
        );
        for offset in iter {
            parts.push(
                InnerPartition {
                    left: prev,
                    right: offset,
                }.join(self)
            );
            prev = offset;
        }
        parts.push(
            LastPartition {
                inner: prev,
            }.join(self)
        );
        parts
    }
}