use crate::*;

#[derive(Debug, Deref, DerefMut)]
pub struct RangeMap {
    #[deref]
    map: HashMap<Range<usize>, Child>,
}
impl<C: Borrow<Child>, I: IntoIterator<Item=C>> From<I> for RangeMap {
    fn from(iter: I) -> Self {
        let mut map = HashMap::default();
        for (i, part) in iter.into_iter().enumerate() {
            map.insert(
                i..i, 
                *part.borrow(),
            );
        }
        Self { map }
    }
}
impl RangeMap {
    pub fn range_sub_merges(&self, range: Range<usize>) -> impl IntoIterator<Item=Pattern> + '_ {
        let (start, end) = (range.start, range.end);
        range.into_iter()
            .map(move |ri| {
                let &left = self.map.get(&(start..ri))
                    .unwrap();
                let &right = self.map.get(&(ri..end))
                    .unwrap();
                vec![left, right]
            })
    }
}
impl<'p> JoinContext<'p> {
    pub fn index_partitions(
        &mut self,
        sub_splits: impl HasSubSplits,
    ) -> Vec<Child> {
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
            ((), prev).join_partition(self).into()
        );
        for offset in iter {
            parts.push(
                (prev, offset).join_partition(self).into()
            );
            prev = offset;
        }
        parts.push(
            (prev, ()).join_partition(self).into()
        );
        parts
    }
    pub fn indexed_partition_patterns<'a, K: RangeRole, P: AsPartition<'a, K>>(
        &mut self,
        part: P,
    ) -> Result<JoinedPatterns<K>, Child>
        where 'p: 'a
    {
        part.info_bundle(self)
            .map(|b| b.join_patterns(self))
    }
}
impl<'p> JoinContext<'p> {
    pub fn merge_partitions(
        &mut self,
        offsets: &mut BTreeMap<NonZeroUsize, OffsetSplits>,
        partitions: &Vec<Child>,
    ) -> RangeMap {
        let mut range_map = RangeMap::from(partitions);
        let num_offsets = offsets.len();
        for len in 1..num_offsets {
            for start in 0..num_offsets-len+1 {
                let range = start..start + len;
                
                // create PartitionBundle
                let lo = offsets.iter().nth(start).unwrap();
                let ro = offsets.iter().nth(start + len).unwrap();

                let infos = (lo, ro).info_bundle(self);

                // how to handle full index?
                let index = match infos {
                    Ok(bundle) => {
                        let merges = range_map.range_sub_merges(start..start + len);
                        
                        //let patterns = merges.into_iter().chain(
                        //    bundle.patterns.into_iter().map(|p|
                        //        (p.borrow() as &[Child]).into_iter().cloned().collect_vec()
                        //    )
                        //);
                        //let index = self.graph_mut().insert_patterns(
                        //    patterns
                        //);
                        // todo: make sure to build new perfect partitions correctly
                        index
                    },
                    Err(part) => part,
                };
                range_map.insert(
                    range,
                    index,
                );
            } 
        }
        range_map
    }
    pub fn merge_node(
        &mut self,
        partitions: &Vec<Child>,
    ) -> LinkedHashMap<SplitKey, Split> {
        let mut offsets = self.sub_splits_mut();
        assert!(partitions.len() == offsets.len() + 1);

        let merges = self.merge_partitions(
            offsets,
            &partitions,
        );

        let len = offsets.len();
        let mut finals = LinkedHashMap::new();
        for (i, (offset, v)) in offsets.iter().enumerate() {
            let lr = 0..i;
            let rr = i+1..len;
            let left = *merges.get(&lr).unwrap();
            let right = *merges.get(&rr).unwrap();
            if !lr.is_empty() || !lr.is_empty() {
                if let Some((&pid, _)) = v.borrow().iter().find(|(_, s)| s.inner_offset.is_none()) {
                    self.graph_mut().replace_in_pattern(index.to_pattern_location(pid), 0.., [left, right]);
                } else {
                    self.graph_mut().add_pattern_with_update(index, [left, right]);
                }
            }
            finals.insert(SplitKey::new(index, *offset), Split::new(left, right));
        }
        finals
    }
}