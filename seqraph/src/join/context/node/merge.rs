use std::{
    borrow::Borrow,
    ops::Range,
};

use derive_more::{
    Deref,
    DerefMut,
};
use itertools::Itertools;
use linked_hash_map::LinkedHashMap;

use crate::{
    HashMap,
    join::{
        context::node::context::NodeJoinContext,
        partition::{
            Infix,
            info::{
                PartitionInfo,
                range::role::{
                    In,
                    Join,
                },
                visit::VisitPartition,
            },
            splits::{
                HasPosSplits,
                PosSplits,
                SplitKind,
            },
        },
    },
    split::cache::split::Split,
    traversal::cache::key::SplitKey,
};
use crate::graph::vertex::{
    child::Child,
    pattern::Pattern,
};

impl<'p> NodeJoinContext<'p> {
    //pub fn indexed_partition_patterns<'a, K: RangeRole<Mode = Join> + 'p, P: AsPartition<K>>(
    //    &mut self,
    //    part: P,
    //) -> Result<JoinedPatterns<K>, Child>
    //    where 'p: 'a,
    //        &'a K::Splits: AsPartition<K>
    //{
    //    part.visit_partition(&*self)
    //        .map(|b| b.join_patterns(self))
    //}
    pub fn merge_node<S: HasPosSplits>(
        &mut self,
        splits: S,
        partitions: &Vec<Child>,
    ) -> LinkedHashMap<SplitKey, Split>
    where
        for<'t> &'t S::Split: SplitKind,
        PosSplits<S>: HasPosSplits<Split = S::Split>,
    {
        let offsets = splits.pos_splits();
        assert_eq!(partitions.len(), offsets.len() + 1);

        let merges = self.merge_partitions(offsets, &partitions);

        let len = offsets.len();
        let index = self.index;
        let mut finals = LinkedHashMap::new();
        for (i, (offset, v)) in offsets.iter().enumerate() {
            let lr = 0..i;
            let rr = i + 1..len;
            let left = *merges.get(&lr).unwrap();
            let right = *merges.get(&rr).unwrap();
            if !lr.is_empty() || !lr.is_empty() {
                if let Some((&pid, _)) = v.borrow().iter().find(|(_, s)| s.inner_offset.is_none()) {
                    self.graph.replace_in_pattern(
                        index.to_pattern_location(pid),
                        0..,
                        [left, right],
                    );
                } else {
                    self.graph.add_pattern_with_update(index, [left, right]);
                }
            }
            finals.insert(SplitKey::new(index, *offset), Split::new(left, right));
        }
        finals
    }
    pub fn merge_partitions<S: HasPosSplits>(
        &mut self,
        splits: S,
        partitions: &Vec<Child>,
    ) -> RangeMap
    where
        for<'t> &'t S::Split: SplitKind,
        PosSplits<S>: HasPosSplits<Split = S::Split>,
    {
        let offsets = splits.pos_splits();
        let num_offsets = offsets.len();

        let mut range_map = RangeMap::from(partitions);

        for len in 1..num_offsets {
            for start in 0..num_offsets - len + 1 {
                let range = start..start + len;

                let lo = offsets.iter().nth(start).unwrap();
                let ro = offsets.iter().nth(start + len).unwrap();

                // todo: could be read from cache
                let res: Result<PartitionInfo<In<Join>>, _> =
                    Infix::new(lo, ro).info_partition(self);

                let index = match res {
                    Ok(info) => {
                        let merges = range_map.range_sub_merges(start..start + len);
                        let joined = info.patterns.into_iter().map(|(pid, info)| {
                            (info.joined_pattern(self, &pid).borrow() as &'_ Pattern)
                                .into_iter()
                                .cloned()
                                .collect_vec()
                        });
                        // todo: insert into perfect context
                        let patterns = merges.into_iter().chain(joined).collect_vec();
                        self.graph.insert_patterns(patterns)
                    }
                    Err(c) => c,
                };
                range_map.insert(range, index);
            }
        }
        range_map
    }
}

#[derive(Debug, Deref, DerefMut, Default)]
pub struct RangeMap<R = Range<usize>> {
    #[deref]
    map: HashMap<R, Child>,
}

impl<C: Borrow<Child>, I: IntoIterator<Item = C>> From<I> for RangeMap<Range<usize>> {
    fn from(iter: I) -> Self {
        let mut map = HashMap::default();
        for (i, part) in iter.into_iter().enumerate() {
            map.insert(i..i, *part.borrow());
        }
        Self { map }
    }
}

impl RangeMap<Range<usize>> {
    pub fn range_sub_merges(
        &self,
        range: Range<usize>,
    ) -> impl IntoIterator<Item = Pattern> + '_ {
        let (start, end) = (range.start, range.end);
        range.into_iter().map(move |ri| {
            let &left = self.map.get(&(start..ri)).unwrap();
            let &right = self.map.get(&(ri..end)).unwrap();
            vec![left, right]
        })
    }
}
