use crate::*;

pub mod splits;
pub use splits::*;
pub mod info;
pub use info::*;

#[derive(new, Clone, Copy)]
pub struct Infix<
    A: AsOffsetSplits,
    B: AsOffsetSplits,
> {
    pub left: A,
    pub right: B,
}
#[derive(new, Clone)]
pub struct Prefix<O: AsOffsetSplits> {
    pub split: O,
}
#[derive(new, Clone)]
pub struct Postfix<O: AsOffsetSplits> {
    pub split: O,
}

#[derive(Debug, Clone)]
pub struct Partition<K: RangeRole> {
    pub offsets: K::Splits,
}
//pub trait IntoPartition<K: RangeRole> {
//    fn into_partition<'p>(self, ctx: &'p mut Partitioner<'p>) -> Partition<K>;
//}
//
//impl<K: RangeRole> IntoPartition<K> for <K::Kind as RangeKind>::Offsets {
//    fn into_partition<'p>(self, ctx: &'p mut Partitioner<'p>) -> Partition<K> {
//        Partition {
//            offsets: self.as_splits(ctx),
//        }
//    }
//}
pub trait AsPartition<K: RangeRole>: VisitPartition<K> {
    fn as_partition(self) -> Partition<K>;
}
impl<K: RangeRole> AsPartition<K> for Partition<K> {
    fn as_partition(self) -> Partition<K> {
        self
    }
}
impl<'a, M: InVisitMode, A: AsOffsetSplits, B: AsOffsetSplits> AsPartition<In<M>> for Infix<A, B> {
    fn as_partition(self) -> Partition<In<M>> {
        Partition {
            offsets: (
                self.left.as_offset_splits(),
                self.right.as_offset_splits(),
            ),
        }
    }
}
impl<'a, M: InVisitMode> AsPartition<In<M>> for (OffsetSplits, OffsetSplits) {
    fn as_partition(self) -> Partition<In<M>> {
        Partition {
            offsets: (
                self.0,
                self.1,
            )
        }
    }
}
impl<'a, M: InVisitMode> AsPartition<In<M>> for &'a (OffsetSplits, OffsetSplits) {
    fn as_partition(self) -> Partition<In<M>> {
        Partition {
            offsets: (
                self.0.clone(),
                self.1.clone(),
            )
        }
    }
}
impl<'a, M: PreVisitMode, A: AsOffsetSplits> AsPartition<Pre<M>> for A {
    fn as_partition(self) -> Partition<Pre<M>> {
        Partition {
            offsets: self.as_offset_splits(),
        }
    }
}
impl<'a, M: PostVisitMode, A: AsOffsetSplits> AsPartition<Post<M>> for A {
    fn as_partition(self) -> Partition<Post<M>> {
        Partition {
            offsets: self.as_offset_splits(),
        }
    }
}
impl<'a, M: PreVisitMode, B: AsOffsetSplits> AsPartition<Pre<M>> for Prefix<B> {
    fn as_partition(self) -> Partition<Pre<M>> {
        Partition {
            offsets: self.split.as_offset_splits(),
        }
    }
}
impl<'a, M: PostVisitMode, A: AsOffsetSplits> AsPartition<Post<M>> for Postfix<A> {
    fn as_partition(self) -> Partition<Post<M>> {
        Partition {
            offsets: self.split.as_offset_splits(),
        }
    }
}

pub fn to_non_zero_range(
    l: usize,
    r: usize,
) -> (NonZeroUsize, NonZeroUsize) {
    (
        NonZeroUsize::new(l).unwrap(),
        NonZeroUsize::new(r).unwrap(),
    )
}
//impl Indexer {
//    fn get_partition(
//        &mut self,
//        merges: &HashMap<Range<usize>, Child>,
//        offsets: &Vec<Offset>,
//        range: &Range<usize>,
//    ) -> Option<Child> {
//        let graph = self.graph();
//        //let split_map: BTreeMap<_, Split<Option<Child>>> = Default::default();
//
//        let wrapper = merges.get(range)?;
//        Some(if range.start == range.end {
//            *wrapper
//        } else {
//            let pre_width = range.start.checked_sub(1)
//                .map(|prev| NonZeroUsize::new(
//                    offsets[range.start].get() - offsets[prev].get()
//                ).unwrap())
//                .unwrap_or(offsets[range.start]);
//
//            let wrapper = merges.get(range)?;
//            let node = graph.expect_vertex_data(wrapper);
//
//            let (_, pat) = node.get_child_pattern_with_prefix_width(pre_width).unwrap();
//
//            let wrapper2 = pat[1];
//            let node2 = graph.expect_vertex_data(wrapper2);
//
//
//            let inner_width = NonZeroUsize::new(offsets[range.end].get() - offsets[range.start].get()).unwrap();
//            let (_, pat2) = node2.get_child_pattern_with_prefix_width(inner_width).unwrap();
//            pat2[0]
//        })
//    }
//}
#[cfg(tests)]
mod tests {
    fn first_partition() {

    }
    fn inner_partition() {
        let cache = SplitCache {
            entries: HashMap::from([]),
            leaves: vec![],
        };
        let patterns = vec![

        ];
        let (lo, ro) = to_non_zero_range(1, 3);
        let (ls, rs) = range_splits(&patterns, (lo, ro));
        let (l, r) = ((&lo, ls), (&ro, rs)); 
        let bundle = (l, r).info_bundle();
    }
    fn last_partition() {

    }
}