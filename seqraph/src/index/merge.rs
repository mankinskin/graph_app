use crate::*;

impl Indexer {
    pub fn merge_node(
        &mut self,
        index: &Child,
        partitions: &Vec<Child>,
        cache: &mut SplitCache,
    ) {
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
    pub fn merge_partitions(
        &mut self,
        partitions: &Vec<Child>,
    ) -> HashMap<Range<usize>, Child> {
        let mut graph = self.graph_mut();
        //let split_map: BTreeMap<_, Split<Option<Child>>> = Default::default();

        // this will contain all future indices
        let mut range_map = HashMap::default();

        let num_offsets = partitions.len() - 1;
        for (i, part) in partitions.iter().enumerate() {
            range_map.insert(
                i..i, 
                *part,
            );
        }
        for len in 1..num_offsets {
            for start in 0..num_offsets-len+1 {
                let range = start..start + len;
                
                let patterns = 
                    (start..start + len).into_iter()
                        .map(|ri| {
                            let &left = range_map.get(&(start..ri))
                                .unwrap();
                            let &right = range_map.get(&(ri..start + len))
                                .unwrap();
                            vec![left, right]
                        });
                range_map.insert(
                    range,
                    graph.insert_patterns(
                        patterns
                    ),
                );
            } 
        }
        range_map
    }
}