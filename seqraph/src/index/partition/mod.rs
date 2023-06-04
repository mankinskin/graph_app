use crate::*;

pub mod bundle;
pub use bundle::*;
pub mod partitioner;
pub use partitioner::*;
pub mod splits;
pub use splits::*;
pub mod info;
pub use info::*;

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