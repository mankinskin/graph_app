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
impl<'p> Partitioner<'p> {
    pub fn merge_partitions(
        &mut self,
        offsets: &mut BTreeMap<NonZeroUsize, SplitPositionCache>,
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

                let joined = self.indexed_partition_patterns((lo, ro));

                // how to handle full index?
                let index = match joined {
                    Ok(bundle) => {
                        let merges = range_map.range_sub_merges(start..start + len);
                        let patterns = merges.into_iter().chain(
                            bundle.patterns.into_iter().map(|p|
                                (p.borrow() as &[Child]).into_iter().cloned().collect_vec()
                            )
                        );
                        let index = self.graph.insert_patterns(
                            patterns
                        );
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
        let index = self.index.index;
        let mut vert_cache = (*self.cache.entries.get(&index).unwrap()).clone();
        let mut offsets = &mut vert_cache.positions;
        assert!(partitions.len() == offsets.len() + 1);

        let merges = self.merge_partitions(
            &mut offsets,
            &partitions,
        );

        let len = offsets.len();
        let index = self.index;
        let mut finals = LinkedHashMap::new();
        for (i, (offset, v)) in offsets.iter().enumerate() {
            let lr = 0..i;
            let rr = i+1..len;
            let left = *merges.get(&lr).unwrap();
            let right = *merges.get(&rr).unwrap();
            if !lr.is_empty() || !lr.is_empty() {
                if let Some((&pid, _)) = v.pattern_splits.iter().find(|(_, s)| s.inner_offset.is_none()) {
                    self.ctx.graph.replace_in_pattern(index.to_pattern_location(pid), 0.., [left, right]);
                } else {
                    self.ctx.graph.add_pattern_with_update(index, [left, right]);
                }
            }
            finals.insert(SplitKey::new(index, *offset), Split::new(left, right));
        }
        finals
    }
}