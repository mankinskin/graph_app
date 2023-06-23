use crate::*;

pub mod splits;
pub use splits::*;
pub mod info;
pub use info::*;

#[derive(Debug, Default)]
pub struct PartitionInfo<K: RangeRole> {
    pub patterns: HashMap<PatternId, RangeInfo<K>>,
    pub perfect: K::Perfect,
}

impl<K: RangeRole> FromIterator<PatternRangeInfo<K>> for PartitionInfo<K> {
    fn from_iter<T: IntoIterator<Item = PatternRangeInfo<K>>>(iter: T) -> Self {
        let mut perf = K::Perfect::default();
        let patterns =
            iter.into_iter()
                .map(|PatternRangeInfo {
                    pattern_id,
                    info,
                    perfect,
                }| {
                    perf.fold_or(perfect.then_some(pattern_id));
                    (pattern_id, info)
                })
                .collect();
        PartitionInfo {
            patterns,
            perfect: perf,
            //delta,
        }
    }
}
impl<'a, K: RangeRole<Mode = Join>> PartitionInfo<K> {
    pub fn join_patterns(
        self,
        ctx: &mut JoinContext<'a>,
    ) -> JoinedPatterns<K>
    {
        let (delta, patterns) = self.patterns.into_iter()
            .map(|(pattern_id, info)|
                (
                    (pattern_id, info.delta),
                    info.join_pattern_inner(pattern_id, ctx),
                )
            )
            .unzip();
        JoinedPatterns {
            patterns,
            perfect: self.perfect,
            delta,
        }
    }
    pub fn join(
        self,
        ctx: &mut JoinContext<'a>,
    ) -> JoinedPartition<K>
    {
        // collect infos about partition in each pattern
        self.join_patterns(ctx).join(ctx)
    }
}
impl Indexer {
    fn get_partition(
        &mut self,
        merges: &HashMap<Range<usize>, Child>,
        offsets: &Vec<Offset>,
        range: &Range<usize>,
    ) -> Option<Child> {
        let graph = self.graph();
        //let split_map: BTreeMap<_, Split<Option<Child>>> = Default::default();

        let wrapper = merges.get(range)?;
        Some(if range.start == range.end {
            *wrapper
        } else {
            let pre_width = range.start.checked_sub(1)
                .map(|prev| NonZeroUsize::new(
                    offsets[range.start].get() - offsets[prev].get()
                ).unwrap())
                .unwrap_or(offsets[range.start]);

            let wrapper = merges.get(range)?;
            let node = graph.expect_vertex_data(wrapper);

            let (_, pat) = node.get_child_pattern_with_prefix_width(pre_width).unwrap();

            let wrapper2 = pat[1];
            let node2 = graph.expect_vertex_data(wrapper2);


            let inner_width = NonZeroUsize::new(offsets[range.end].get() - offsets[range.start].get()).unwrap();
            let (_, pat2) = node2.get_child_pattern_with_prefix_width(inner_width).unwrap();
            pat2[0]
        })
    }
}